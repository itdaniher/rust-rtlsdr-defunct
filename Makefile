all:
	rustc -A unused-variable --opt-level=3 --link-args '-lrtlsdr -lkissfft -lpulse -lpulse-simple' -L./ rtlsdr.rc

clean:
	rm rtlsdr
