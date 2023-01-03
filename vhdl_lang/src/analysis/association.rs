#![allow(clippy::only_used_in_recursion)]

use fnv::FnvHashSet;

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2018, Olof Kraigher olof.kraigher@gmail.com
use super::analyze::*;
use super::formal_region::FormalRegion;
use super::formal_region::InterfaceEnt;
use super::region::*;
use crate::ast::*;
use crate::data::*;

pub enum ResolvedFormal {
    // A basic formal
    // port map(foo => 0)
    Basic(usize, InterfaceEnt),

    /// A formal that is either selected such as a record field of array index
    /// Example:
    /// port map(foo.field => 0)
    /// port map(foo(0) => 0)
    Selected(usize, InterfaceEnt, TypeEnt),

    /// A formal that has been converted by a function
    /// Could also be a converted selected formal
    /// Example:
    /// port map(to_slv(foo) => sig)
    Converted(usize, InterfaceEnt, TypeEnt),
}

impl ResolvedFormal {
    pub fn base_type(&self) -> &TypeEnt {
        self.type_mark().base_type()
    }

    pub fn type_mark(&self) -> &TypeEnt {
        match self {
            ResolvedFormal::Basic(_, ent) => ent.type_mark(),
            ResolvedFormal::Selected(_, _, typ) => typ,
            ResolvedFormal::Converted(_, _, typ) => typ,
        }
    }

    fn select(self, suffix_type: TypeEnt) -> Option<Self> {
        match self {
            ResolvedFormal::Basic(idx, ent) => {
                Some(ResolvedFormal::Selected(idx, ent, suffix_type))
            }
            ResolvedFormal::Selected(idx, ent, _) => {
                Some(ResolvedFormal::Selected(idx, ent, suffix_type))
            }

            // Converted formals may not be further selected
            ResolvedFormal::Converted(..) => None,
        }
    }

    // The position of the formal in the formal region
    fn idx(&self) -> usize {
        *match self {
            ResolvedFormal::Basic(idx, _) => idx,
            ResolvedFormal::Selected(idx, _, _) => idx,
            ResolvedFormal::Converted(idx, _, _) => idx,
        }
    }
}

impl<'a> AnalyzeContext<'a> {
    pub fn resolve_formal(
        &self,
        formal_region: &FormalRegion,
        region: &Region<'_>,
        name_pos: &SrcPos,
        name: &mut Name,
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> AnalysisResult<ResolvedFormal> {
        match name {
            Name::Selected(prefix, suffix) => {
                suffix.clear_reference();

                let resolved_prefix = self.resolve_formal(
                    formal_region,
                    region,
                    &prefix.pos,
                    &mut prefix.item,
                    diagnostics,
                )?;

                let suffix_ent =
                    self.lookup_type_selected(&prefix.pos, resolved_prefix.type_mark(), suffix)?;
                suffix.set_reference(&suffix_ent);

                let suffix_ent = suffix_ent
                    .expect_non_overloaded(&suffix.pos, || "Invalid formal".to_string())?;

                suffix.set_unique_reference(&suffix_ent);

                if let NamedEntityKind::ElementDeclaration(elem) = suffix_ent.actual_kind() {
                    if let Some(resolved_formal) = resolved_prefix.select(elem.type_mark().clone())
                    {
                        Ok(resolved_formal)
                    } else {
                        Err(Diagnostic::error(name_pos, "Invalid formal").into())
                    }
                } else {
                    Err(Diagnostic::error(name_pos, "Invalid formal").into())
                }
            }

            Name::SelectedAll(_) => Err(Diagnostic::error(name_pos, "Invalid formal").into()),
            Name::Designator(designator) => {
                designator.clear_reference();
                let (idx, ent) = formal_region.lookup(name_pos, designator.designator())?;
                designator.set_unique_reference(ent.inner());
                Ok(ResolvedFormal::Basic(idx, ent))
            }
            Name::Indexed(ref mut prefix, ref mut indexes) => {
                let resolved_prefix = self.resolve_formal(
                    formal_region,
                    region,
                    &prefix.pos,
                    &mut prefix.item,
                    diagnostics,
                )?;

                let new_typ = self.analyze_indexed_name(
                    region,
                    name_pos,
                    prefix.suffix_pos(),
                    resolved_prefix.type_mark(),
                    indexes,
                    diagnostics,
                )?;

                if let Some(resolved_formal) = resolved_prefix.select(new_typ) {
                    Ok(resolved_formal)
                } else {
                    Err(Diagnostic::error(name_pos, "Invalid formal").into())
                }
            }

            Name::Slice(ref mut prefix, ref mut drange) => {
                let resolved_prefix = self.resolve_formal(
                    formal_region,
                    region,
                    &prefix.pos,
                    &mut prefix.item,
                    diagnostics,
                )?;

                if let ResolvedFormal::Converted(..) = resolved_prefix {
                    // Converted formals may not be further selected
                    return Err(Diagnostic::error(name_pos, "Invalid formal").into());
                }

                self.analyze_discrete_range(region, drange.as_mut(), diagnostics)?;
                Ok(resolved_prefix)
            }
            Name::Attribute(..) => Err(Diagnostic::error(name_pos, "Invalid formal").into()),
            Name::FunctionCall(ref mut fcall) => {
                let prefix = if let Some(prefix) = fcall.name.item.prefix() {
                    prefix
                } else {
                    return Err(Diagnostic::error(name_pos, "Invalid formal").into());
                };

                if formal_region.lookup(name_pos, prefix.designator()).is_err() {
                    // The prefix of the name was not found in the formal region
                    // it must be a type conversion or a single parameter function call

                    let (idx, formal_ent) = if let Some(designator) =
                        to_formal_conversion_argument(&mut fcall.parameters)
                    {
                        designator.clear_reference();
                        let (idx, ent) = formal_region.lookup(name_pos, designator.designator())?;
                        designator.set_unique_reference(ent.inner());
                        (idx, ent)
                    } else {
                        return Err(Diagnostic::error(name_pos, "Invalid formal conversion").into());
                    };

                    let converted_typ = match self.resolve_name(
                        region,
                        &fcall.name.pos,
                        &mut fcall.name.item,
                        diagnostics,
                    )? {
                        Some(NamedEntities::Single(ent)) => {
                            // @TODO check type conversion is legal
                            TypeEnt::from_any(ent).map_err(|_| {
                                Diagnostic::error(
                                    name_pos,
                                    "Invalid formal conversion, expected function",
                                )
                            })?
                        }
                        Some(NamedEntities::Overloaded(overloaded)) => {
                            let mut candidates = Vec::with_capacity(overloaded.len());

                            for ent in overloaded.entities() {
                                if let Some(sig) = ent.signature() {
                                    if let Some(rtype) = sig.return_type() {
                                        if sig.can_be_called_with_single_parameter(
                                            formal_ent.type_mark(),
                                        ) {
                                            candidates.push((ent, rtype));
                                        }
                                    }
                                }
                            }

                            if candidates.len() > 1 {
                                // Ambiguous call
                                let mut diagnostic = Diagnostic::error(
                                    &fcall.name.pos,
                                    format!("Ambiguous call to function '{}'", fcall.name),
                                );

                                diagnostic.add_subprogram_candidates(
                                    "migth be",
                                    candidates
                                        .into_iter()
                                        .map(|(ent, _)| ent)
                                        .collect::<Vec<_>>(),
                                );

                                return Err(diagnostic.into());
                            } else if let Some((_, rtype)) = candidates.pop() {
                                rtype.clone()
                            } else {
                                // No match
                                return Err(Diagnostic::error(
                                    &fcall.name.pos,
                                    format!(
                                        "No function '{}' accepting {}",
                                        fcall.name,
                                        formal_ent.type_mark().describe()
                                    ),
                                )
                                .into());
                            }
                        }
                        None => {
                            return Err(
                                Diagnostic::error(name_pos, "Invalid formal conversion").into()
                            );
                        }
                    };

                    Ok(ResolvedFormal::Converted(idx, formal_ent, converted_typ))
                } else if let Some((prefix, indexes)) = fcall.to_indexed() {
                    *name = Name::Indexed(prefix, indexes);
                    self.resolve_formal(formal_region, region, name_pos, name, diagnostics)
                } else {
                    Err(Diagnostic::error(name_pos, "Invalid formal").into())
                }
            }
            Name::External(..) => Err(Diagnostic::error(name_pos, "Invalid formal").into()),
        }
    }

    pub fn analyze_assoc_elems_with_formal_region(
        &self,
        error_pos: &SrcPos, // The position of the instance/call-site
        formal_region: &FormalRegion,
        region: &Region<'_>,
        elems: &mut [AssociationElement],
        diagnostics: &mut dyn DiagnosticHandler,
    ) -> FatalResult<Option<bool>> {
        let mut is_correct = Some(true);

        let mut associated_indexes: FnvHashSet<usize> = Default::default();
        let mut extra_associations: Vec<&SrcPos> = Default::default();

        for (idx, AssociationElement { formal, actual }) in elems.iter_mut().enumerate() {
            let formal_ent = if let Some(ref mut formal) = formal {
                // Call by name using formal
                match self.resolve_formal(
                    formal_region,
                    region,
                    &formal.pos,
                    &mut formal.item,
                    diagnostics,
                ) {
                    Err(err) => {
                        err.add_to(diagnostics)?;
                        is_correct = Some(false);
                        None
                    }
                    Ok(formal) => Some(formal),
                }
            } else if let Some(formal) = formal_region.nth(idx) {
                Some(ResolvedFormal::Basic(idx, formal.clone()))
            } else {
                extra_associations.push(&actual.pos);
                None
            };

            if let Some(formal_ent) = formal_ent {
                associated_indexes.insert(formal_ent.idx());

                match actual.item {
                    ActualPart::Expression(ref mut expr) => {
                        let actual_is_correct = self.analyze_expression_with_target_type(
                            region,
                            formal_ent.base_type(),
                            &actual.pos,
                            expr,
                            diagnostics,
                        )?;

                        if actual_is_correct == Some(false) {
                            is_correct = Some(false);
                        } else if actual_is_correct.is_none() && is_correct == Some(true) {
                            is_correct = None;
                        }
                    }
                    ActualPart::Open => {}
                }
            } else {
                // Formal not found, analyze expressions anyway to get hover references
                match actual.item {
                    ActualPart::Expression(ref mut expr) => {
                        self.analyze_expression_pos(region, &actual.pos, expr, diagnostics)?;
                    }
                    ActualPart::Open => {}
                }
            }
        }

        let mut not_associated = Vec::new();
        for (idx, formal) in formal_region.iter().enumerate() {
            if !associated_indexes.contains(&idx) && !formal.has_default() {
                not_associated.push(idx);
            }
        }

        if not_associated.is_empty() && extra_associations.is_empty() {
            Ok(is_correct)
        } else {
            if is_correct == Some(true) {
                // Only complain if nothing else is wrong
                for idx in not_associated {
                    if let Some(formal) = formal_region.nth(idx) {
                        if formal_region.typ == InterfaceListType::Port && formal.is_output_signal()
                        {
                            // Output ports are allowed to be unconnected
                            continue;
                        }

                        let mut diagnostic = Diagnostic::error(
                            error_pos,
                            format!("No association of {}", formal.describe()),
                        );

                        if let Some(decl_pos) = formal.decl_pos() {
                            diagnostic.add_related(decl_pos, "Defined here");
                        }

                        diagnostics.push(diagnostic);
                    }
                }
                for pos in extra_associations.into_iter() {
                    diagnostics.error(pos, "Unexpected extra argument")
                }
            }
            Ok(Some(false))
        }
    }
}

fn to_formal_conversion_argument(
    parameters: &mut [AssociationElement],
) -> Option<&mut WithRef<Designator>> {
    if let &mut [AssociationElement {
        ref formal,
        ref mut actual,
    }] = parameters
    {
        if formal.is_some() {
            return None;
        } else if let ActualPart::Expression(Expression::Name(ref mut actual_name)) = actual.item {
            if let Name::Designator(designator) = actual_name.as_mut() {
                return Some(designator);
            }
        }
    }
    None
}