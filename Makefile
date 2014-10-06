all: main.rs
	rustc -A dead_code $<

clean:
	rm -f main
