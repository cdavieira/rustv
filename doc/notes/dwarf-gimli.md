# gimli/dwarf
I recommend reading [this example from the gimli
repository](https://github.com/gimli-rs/gimli/blob/master/crates/examples/src/bin/simple_write.rs)
to understand how to integrate gimli and the object crates.

Whenever encountering an unknown word/definition, search for that in the
DWARF-5 documentation (Sections 2-6 cover everything you need to possibly know)
> You can find a link to download this documentation in the 'references' section


## About DWARF-5
DIEs - debugging information entries

A DIE (also known simply as entry) is made of:
* 1 Tag
* 1 or more (unique) attributes

There are some important debugging sections found in ELF:
* .debug_abbrev: abbreviations/aliases used in the .debug_info section
* .debug_info: Core DWARF information section (Filename, Build Directory, File
conding language, ...)
* .debug_line: Line number information
* .debug_line_str: ?
* .debug_rnglists: Address ranges on the ELF file (in the DWARF section)
associated with code written in human-language (Read section 2.17 for more info)


## About gimli
You'll commonly see attributes written with the 'DW_AT' prefix. According to
the DWARF-5 docs (Section 2.2), that prefix probably stands for
'DW_ATTRIBUTETYPE'

The DW_AT_ranges is a Non-contiguous range of code addresses. This is the piece
of your ELF file which contains the data for the associated assembly code. It
needs a starting address and a length (in bytes). For my program, the address
of the text section and its size in bytes could be provided.

the gimli code example says that the DW_AT_stmt_lists gets automatically
generated. This attribute holds a section offset to the line number information
for that compilation unit (See 3.1.1 section)

DW_AT_entry_pc is the address of the first executable instruction of the unit

DW_AT_subprogram describes a subroutine or a function (DW_AT_entry_point can
also be used to just indicate the entry point) (See section 3.3)

## the official object + gimli code example
Available at [object + gimli integration -
Simple Example](https://github.com/gimli-rs/gimli/blob/master/crates/examples/src/bin/simple_write.rs)

```
git clone https://github.com/gimli-rs/gimli
cargo run --bin simple_write
gcc -o hello  hello.o -z noexecstack
gdb hello
readelf -a hello.o
```

## references
* [gimli - write module - rs Docs](https://docs.rs/gimli/latest/gimli/write/index.html)
* [object + gimli integration - Simple Example](https://github.com/gimli-rs/gimli/blob/master/crates/examples/src/bin/simple_write.rs)
* [DWARF-4 Sections](https://developer.ibm.com/articles/au-dwarf-debug-format/)
* [DWARF-5 Differences](https://dwarfstd.org/dwarf5std.html)
* [DWARF Documentation](https://dwarfstd.org/download.html)
* [DWARF Introduction](https://swatinem.de/blog/dwarf-lines/)
