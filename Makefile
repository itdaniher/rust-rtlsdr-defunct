all:
	rustc -A unused-variable --opt-level=3 --link-args '-lrtlsdr -lkissfft -lpulse -lpulse-simple' -L/usr/local/lib rtlsdr.rc

clean:
	rm rtlsdr
