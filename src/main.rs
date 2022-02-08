mod xferrep;
use xferrep::{Vc, XStream, XferErr};
use bytes::{BytesMut, BufMut};
use std::vec;


impl XStream for bytes::BytesMut {
	fn out_want<'a>(&mut self, s: usize) -> Result<&'a mut [u8], XferErr> {
	self.reserve(s);
	let p = self.chunk_mut().as_mut_ptr();
	unsafe { self.advance_mut(s); }
	Result::Ok(unsafe { std::slice::from_raw_parts_mut(p, s)})
	//Result::Err(-1)
	}
	fn in_want<'a>(&self, s: usize) -> Result<&'a [u8], XferErr> {
	Result::Err(-1)
	}
}

fn main() {

	let n = Vc::VcNil;
	let iout = Vc::VcInt {i: 12};
	let mut k: Vec<u8> = Vec::new();
	k.push(b'a');

	let sout = Vc::VcStr {s: k};
	//let tmp = [n, iout, sout];
	let mut v: Vec<Vc> = Vec::new();
	v.push(n);
	v.push(iout);
	v.push(sout);
	let subs = Vc::VcVec {vec: v};
	let mut b = BytesMut::new();
	let mut s = 0;
	// s += n.xfer_out(&mut b).unwrap();
	//s += n.xfer_out(&mut b).unwrap();
	//s += iout.xfer_out(&mut b).unwrap();
	//s += sout.xfer_out(&mut b).unwrap();
	s += subs.xfer_out(&mut b).unwrap();
	
println!("OUT {} {:?} ", s, b.freeze());
}
