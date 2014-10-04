all: main.rs
	rustc $<

clean:
	rm main
