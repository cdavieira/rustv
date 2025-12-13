#!/usr/bin/env bash

CROSSPREFIX="riscv32-unknown-linux-gnu-"
LD="${CROSSPREFIX}ld"
OBJDUMP="${CROSSPREFIX}objdump"
READELF="${CROSSPREFIX}readelf"
GDB="${CROSSPREFIX}gdb"
CARGO="cargo"
GDB_SCRIPT="./init.gdb"

exit_if_empty() {
	if [[ -z "$1" ]]; then
		if [[ -n "$2" ]]; then
			echo -e "$2"
		fi
		exit 1
	fi
}

link(){
	exit_if_empty "$1" "Missing object"
	exit_if_empty "$2" "Missing output filename"
	${LD} $1 -o $2
}

examine_elf(){
	exit_if_empty "$1" "Missing object/executable"
	${READELF} -a $1
}

examine_text_section(){
	exit_if_empty "$1" "Missing object/executable"
	${OBJDUMP} -d $1
}

gdb(){
	exit_if_empty "$1" "Missing executable"
	${GDB} -x ${GDB_SCRIPT} $1
}

run_elf(){
	exit_if_empty "$1" "Missing file, provide one of:\n$(ls examples/)"
	${CARGO} run -- --elf-dbg ./examples/$1
}

run_stub(){
	${CARGO} run -- --debugger
}

run_decoder(){
	exit_if_empty "$1" "Missing instruction"
	${CARGO} run -- --decode-bin $1
}

run_builder(){
	exit_if_empty "$1" "Missing file, provide one of:\n$(ls examples/)"
	${CARGO} run -- --build ./examples/$1
}

case "$1" in
	"compile")  run_elf $2 ;;
	"build")    run_builder $2 ;;
	"link")     link $2 $3 ;;
	"stub")     run_stub   ;;
	"decoder")  run_decoder $2 ;;
	"gdb")      gdb $2 ;;
	"readelf")  examine_elf $2 ;;
	"objdump")  examine_text_section $2 ;;
	*) echo "$0 [compile | builder | link | decoder | stub | gdb | readelf | objdump ]"
esac
