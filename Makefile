all:
	rustc --link-args '-lrtlsdr' rtlsdr.rc 

clean:
	rm rtlsdr
