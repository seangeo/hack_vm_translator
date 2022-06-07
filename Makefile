project7.zip: lang.txt VMTranslator.py vm
	zip project7.zip lang.txt VMTranslator.py vm

vm:
	cargo build --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/debug/hack_vmtranslator ./vm
