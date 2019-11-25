// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2019, Olof Kraigher olof.kraigher@gmail.com

use super::*;

#[test]
fn allows_unique_names() {
    let mut builder = LibraryBuilder::new();
    builder.code(
        "libname",
        "
package pkg is
constant a : natural := 0;
constant b : natural := 0;
constant c : natural := 0;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_no_diagnostics(&diagnostics);
}

#[test]
fn forbid_homographs() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
constant a1 : natural := 0;
constant a : natural := 0;
constant a1 : natural := 0;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_subprogram_bodies() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
end package;

package body pkg is
procedure proc(a1, a, a1 : natural) is
constant b1 : natural := 0;
constant b : natural := 0;
constant b1 : natural := 0;

procedure nested_proc(c1, c, c1 : natural) is
  constant d1 : natural := 0;
  constant d : natural := 0;
  constant d1 : natural := 0;
begin
end;

begin
end;
end package body;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(
        diagnostics,
        duplication_diagnostics(&code, &["a1", "b1", "c1", "d1"]),
    );
}

#[test]
fn forbid_homographs_in_component_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
component comp is
generic (
  a1 : natural;
  a : natural;
  a1 : natural
);
port (
  b1 : natural;
  b : natural;
  b1 : natural
);
end component;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_in_record_type_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
type rec_t is record
a1 : natural;
a : natural;
a1 : natural;
end record;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_proteced_type_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
type prot_t is protected
procedure proc(a1, a, a1 : natural);
end protected;

type prot_t is protected body
constant b1 : natural := 0;
constant b : natural := 0;
constant b1 : natural := 0;
end protected body;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_in_subprogram_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
procedure proc(a1, a, a1 : natural);
function fun(b1, a, b1 : natural) return natural;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_in_block() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
begin
blk : block
constant a1 : natural := 0;
constant a : natural := 0;
constant a1 : natural := 0;
begin
process
  constant b1 : natural := 0;
  constant b : natural := 0;
  constant b1 : natural := 0;
begin
end process;
end block;
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_in_process() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
begin
process
constant a1 : natural := 0;
constant a : natural := 0;
constant a1 : natural := 0;
begin
end process;
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_for_generate() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
begin
gen_for: for i in 0 to 3 generate
constant a1 : natural := 0;
constant a : natural := 0;
constant a1 : natural := 0;
begin
process
  constant b1 : natural := 0;
  constant b : natural := 0;
  constant b1 : natural := 0;
begin
end process;
end generate;
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_if_generate() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
begin
gen_if: if true generate
constant a1 : natural := 0;
constant a : natural := 0;
constant a1 : natural := 0;
begin

prcss : process
  constant b1 : natural := 0;
  constant b : natural := 0;
  constant b1 : natural := 0;
begin
end process;

else generate
constant c1 : natural := 0;
constant c: natural := 0;
constant c1 : natural := 0;
begin
prcss : process
  constant d1 : natural := 0;
  constant d : natural := 0;
  constant d1 : natural := 0;
begin
end process;
end generate;
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(
        diagnostics,
        duplication_diagnostics(&code, &["a1", "b1", "c1", "d1"]),
    );
}

#[test]
fn forbid_homographs_case_generate() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
begin
gen_case: case 0 generate
when others =>
  constant a1 : natural := 0;
  constant a : natural := 0;
  constant a1 : natural := 0;
begin
  process
    constant b1 : natural := 0;
    constant b : natural := 0;
    constant b1 : natural := 0;
  begin
  end process;
end generate;
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_in_entity_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
generic (
a1 : natural;
a : natural;
a1 : natural
);
port (
b1 : natural;
b : natural;
b1 : natural
);
constant c1 : natural := 0;
constant c : natural := 0;
constant c1 : natural := 0;
begin

blk : block
constant d1 : natural := 0;
constant d : natural := 0;
constant d1 : natural := 0;
begin

end block;

end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(
        diagnostics,
        duplication_diagnostics(&code, &["a1", "b1", "c1", "d1"]),
    );
}

#[test]
fn forbid_homographs_in_architecture_bodies() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
end entity;

architecture arch of ent is
constant a1 : natural := 0;
constant a : natural := 0;
constant a1 : natural := 0;
begin

blk : block
constant b1 : natural := 0;
constant b : natural := 0;
constant b1 : natural := 0;
begin
end block;

end architecture;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn forbid_homographs_of_type_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
constant a1 : natural := 0;
type a1 is (foo, bar);
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_of_component_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
constant a1 : natural := 0;
component a1 is
port (clk : bit);
end component;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_of_file_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
constant a1 : natural := 0;
file a1 : std.textio.text;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_package_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package gpkg is
generic (foo : natural);
end package;

package pkg is
package a1 is new work.gpkg generic map (foo => bar);
package a1 is new work.gpkg generic map (foo => bar);
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_attribute_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
attribute a1 : string;
attribute a1 : string;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_alias_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
alias a1 is foo;
alias a1 is bar;

-- Legal since subprograms are overloaded
alias b1 is foo[return natural];
alias b1 is bar[return boolean];
end package pkg;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_for_overloaded_vs_non_overloaded() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
alias a1 is foo;
alias a1 is bar[return boolean];

function b1 return natural;
constant b1 : natural := 0;
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1", "b1"]));
}

#[test]
fn enum_literals_may_overload() {
    let mut builder = LibraryBuilder::new();
    builder.code(
        "libname",
        "
package pkg is
type enum_t is (a1, b1);

-- Ok since enumerations may overload
type enum2_t is (a1, b1);
end package;
",
    );

    let diagnostics = builder.analyze();
    check_no_diagnostics(&diagnostics);
}

#[test]
fn forbid_homograph_to_enum_literals() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
type enum_t is (a1, b1);
constant a1 : natural := 0;
function b1 return natural;
end package pkg;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_interface_file_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package pkg is
procedure proc(file a1, a, a1 : std.textio.text);
end package;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_interface_type_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
entity ent is
generic (
type a1;
type a1
);
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_interface_package_declarations() {
    let mut builder = LibraryBuilder::new();
    let code = builder.code(
        "libname",
        "
package gpkg is
generic (const : natural);
end package;

entity ent is
generic (
package a1 is new work.gpkg generic map (const => 0);
package a1 is new work.gpkg generic map (const => 0)
);
end entity;
",
    );

    let diagnostics = builder.analyze();
    check_diagnostics(diagnostics, duplication_diagnostics(&code, &["a1"]));
}

#[test]
fn forbid_homographs_in_entity_extended_declarative_regions() {
    let mut builder = LibraryBuilder::new();
    let ent = builder.code(
        "libname",
        "
entity ent is
generic (
constant g1 : natural;
constant g2 : natural;
constant g3 : natural;
constant g4 : natural
);
port (
signal g1 : natural;
signal p1 : natural;
signal p2 : natural;
signal p3 : natural
);
constant g2 : natural := 0;
constant p1 : natural := 0;
constant e1 : natural := 0;
constant e2 : natural := 0;
end entity;",
    );

    let arch1 = builder.code(
        "libname",
        "
architecture rtl of ent is
constant g3 : natural := 0;
constant p2 : natural := 0;
constant e1 : natural := 0;
constant a1 : natural := 0;
begin
end architecture;",
    );

    let arch2 = builder.code(
        "libname",
        "
architecture rtl2 of ent is
constant a1 : natural := 0;
constant e2 : natural := 0;
begin
end architecture;
",
    );

    let diagnostics = builder.analyze();
    let mut expected = duplication_diagnostics(&ent, &["g1", "g2", "p1"]);
    expected.append(&mut duplication_diagnostics_two_file(
        &ent,
        &arch1,
        &["g3", "p2", "e1"],
    ));
    expected.append(&mut duplication_diagnostics_two_file(&ent, &arch2, &["e2"]));
    check_diagnostics(diagnostics, expected);
}

#[test]
fn forbid_homographs_in_package_extended_declarative_regions() {
    let mut builder = LibraryBuilder::new();
    let pkg = builder.code(
        "libname",
        "
package pkg is
generic (
constant g1 : natural;
constant g2 : natural
);
constant g1 : natural := 0;
end package;",
    );

    let body = builder.code(
        "libname",
        "
package body pkg is
constant g1 : natural := 0;
constant g2 : natural := 0;
constant p1 : natural := 0;
end package body;",
    );

    let diagnostics = builder.analyze();
    let mut expected = duplication_diagnostics(&pkg, &["g1"]);
    expected.append(&mut duplication_diagnostics_two_file(
        &pkg,
        &body,
        &["g1", "g2"],
    ));
    check_diagnostics(diagnostics, expected);
}