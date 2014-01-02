extern mod extra;
extern mod rtlsdr;
extern mod dsputils;
extern mod rle;
extern mod videoSinkSDL1;
use extra::complex;
use extra::sort;
use extra::bitv;
use std::comm::{Port,Chan};

fn validPulse (In: &rle::Run) -> Option<rle::Run> {
	match In.ct {
		10..25000 => Some(In.clone()),
		_ => None
	}
}

fn validSymbolMotion(In: &rle::Run) -> Option<uint> {
	match In.v {
		1 => { match In.ct {
			650..850 => Some(0),
			2100..2300 => Some(1),
			_ => None }},
		0 => { match In.ct {
			22000..24000 => Some(2),
			_ => None }},
		_ => None
	}
}

fn validPulseBert (In: &rle::Run) -> Option<rle::Run> {
		match In.ct {
			40..70 => Some(rle::Run{v:In.v, ct:1}),
			80..140 => Some(rle::Run{v:In.v, ct:2}),
			_ => None
		}
}

fn validBitBert (In: &[uint] ) -> Option<uint> {
	match In.len() { 
		2 => match (In[0], In[1]) {
				(1,0) => Some(1),
				(0,1) => Some(0),
				_ => None
			},
		_ => None
	}
}

fn packetize(In: ~[uint]) -> ~[~[uint]] {
	let mut Out: ~[~[uint]] = ~[];
	let breakSymbol: uint = dsputils::max(In.clone());
	let mut working: ~[uint] = ~[];
	for &x in In.iter() {
		if (x == breakSymbol ) {Out.push(working); working=~[];}
		else {working.push(x)}
	};
	return Out
}
