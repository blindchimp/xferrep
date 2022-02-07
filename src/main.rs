mod xferrep;
use xferrep::{Vc, XStream, XferErr};
use bytes::{BytesMut, BufMut};


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
	let mut b = BytesMut::new();
	let mut s = n.xfer_out(&mut b).unwrap();
	s += n.xfer_out(&mut b).unwrap();
	
println!("OUT {} {:?} ", s, b.freeze());
}
