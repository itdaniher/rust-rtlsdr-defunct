all:
	rustc -A unused-variable --opt-level=3 --link-args '-lrtlsdr -lkissfft' -L./lib -o rtlsdrtest rtlsdr.rc

clean:
	rm rtlsdr
