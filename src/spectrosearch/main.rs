extern mod extra;
extern mod kissfft;
extern mod videoSinkSDL1;
extern mod rtlsdr;
extern mod pa;
extern mod dsputils;

use std::num;
use extra::complex;
use extra::time;

fn main() {
	let centerFreq: f32 = 433.8e6;
	let sRate: f32 = 1.024e6;
	let blockSize: uint = 4096;
	let devHandle = rtlsdr::openDevice(0);
	rtlsdr::setSampleRate(devHandle, sRate as u32);
	rtlsdr::clearBuffer(devHandle);
	//rtlsdr::setGainAuto(devHandle);
	println(format!("{}", centerFreq));
	rtlsdr::setFrequency(devHandle, centerFreq as u32);
	let (p, videoChan) = videoSinkSDL1::spawnVectorVisualSink();
	let pdata = rtlsdr::readAsync(devHandle);
	let bpf = kissfft::kissFFT(dsputils::asRe(dsputils::bpf(blockSize, (433.85e6-centerFreq)/sRate, (433.95e6-centerFreq)/sRate)));
	let (pFFT, cFFT) = kissfft::buildFFTBlock(blockSize as u64, true);
	'main: loop {
		let mut i = 0;
		let mut samples: ~[complex::Complex32] = ~[];
		'accSamples: loop {
			samples = samples + rtlsdr::dataToSamples(pdata.recv());
			if (samples.len() == (blockSize)) {break 'accSamples};
		}
		let datafft = kissfft::kissFFT(samples);
		let ffts = datafft.iter().zip(bpf.iter()).map(|(&x, &y)| x*y).collect();
		let filtered = dsputils::asF32(ffts);
		let f: ~[f32] = filtered.iter().map(|&x| num::abs(x)).enumerate().filter(|&(x, y)| (x > 0)).map(|(x, y)| y).collect();
		let integral: ~[f32] = f.iter().scan(0.0f32, |mut x, &y| {*x += y; Some(*x)}).collect();
		// if the video sink disappears, drop from the loop
		if !videoChan.try_send(dsputils::asF32(datafft)){
			break 'main
		}
		'flush : loop {
			if !pdata.peek() {
				if i > 0 {
 					println(format!("dropped {:?}", i));
				}
 				break 'flush;
 			}
 			else {
				pdata.recv();
				i = i + 1;
 			}
		}
	}
	rtlsdr::stopAsync(devHandle);
	rtlsdr::close(devHandle);
}
