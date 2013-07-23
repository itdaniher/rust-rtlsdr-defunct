all:
	rustc -O --link-args '-lrtlsdr' rtlsdr.rc 

clean:
	rm rtlsdr
