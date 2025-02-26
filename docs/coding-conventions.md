# Move Coding Conventions

To ensure the quality of project code and collaboration among developers, I think we should have a unified coding conventions in this repo.

This section lays out some basic coding conventions for Move that the Move team has found helpful.

## Naming

- **Module names**: should be lower snake case, e.g., `fixed_point32`, `vector`.
- **Type names**: should be camel case if they are not a native type, e.g., `Coin`, `RoleId`.
- **Function names**: should be lower snake case, e.g., `destroy_empty`.
- **Constant names**: should be upper camel case and begin with an `E` if they represent error codes (e.g., `EIndexOutOfBounds`) and upper snake case if they represent a non-error value (e.g., `MIN_STAKE`).
- **Generic type names**: should be descriptive, or anti-descriptive where appropriate, e.g., `T` or `Element` for the Vector generic type parameter. Most of the time the "main" type in a module should be the same name as the module e.g., `option::Option`, `fixed_point32::FixedPoint32`.
- **Module file names**: should be the same as the module name and lower snake case, e.g., `option.move`.
- **Script file names**: should be lower snake case and should match the name of the “main” function in the script.
- **Mixed file names**: If the file contains multiple modules and/or scripts, the file name should be lower snake case, where the name does not match any particular module/script inside.

## Imports

- All module `use` statements should be at the top of the module.
- Functions should be imported and used fully qualified from the module in which they are declared, and not imported at the top level.
- Types should be imported at the top-level. Where there are name clashes, `as` should be used to rename the type locally as appropriate.

For example, if there is a module:

```move
module 0x1::foo {
    struct Foo {}
    const CONST_FOO: u64 = 0;
    public fun do_foo(): Foo { Foo{} }
    ...
}
```

this would be imported and used as:

```move
module 0x1::bar {
    use 0x1::foo::{Self, Foo};

    public fun do_bar(x: u64): Foo {
        if (x == 10) {
            foo::do_foo()
        } else {
            abort 0
        }
    }
    ...
}
```

And, if there is a local name-clash when importing two modules:

```move
module other_foo {
    struct Foo {}
    ...
}

module 0x1::importer {
    use 0x1::other_foo::Foo as OtherFoo;
    use 0x1::foo::Foo;
    ...
}
```

## Comments

- Each module, struct, and public function declaration should be commented, comments should reflect the intent and design of the code .
- Move has doc comments `///`, regular single-line comments `//`, block comments `/* */`, and block doc comments `/** */`.

## Formatting

The Move team plans to write an autoformatter to enforce formatting conventions. However, in the meantime:

- Four space indentation should be used except for `script` and `address` blocks whose contents should not be indented.
- Lines should be broken if they are longer than 100 characters.
- Structs and constants should be declared before all functions in a module.
