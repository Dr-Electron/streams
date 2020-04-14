//! Lightweight abstraction, a trinary equivalent of `Write` trait allowing access to trinary slices.

use failure::{
    bail,
    ensure,
    Fallible,
};
use iota_streams_core::tbits::{
    word::BasicTbitWord,
    TbitSlice,
    TbitSliceMut,
};

/// Write
pub trait OStream<TW> {
    /// Try advance and panic in case of error.
    fn advance<'a>(&'a mut self, n: usize) -> TbitSliceMut<'a, TW> {
        let r = self.try_advance(n);
        assert!(r.is_ok());
        r.unwrap()
    }

    /// Try put n tbits into the stream, returning a slice to the buffer.
    fn try_advance<'a>(&'a mut self, n: usize) -> Fallible<TbitSliceMut<'a, TW>>;

    /// Commit advanced buffers to the internal sink.
    fn commit(&mut self);

    /// Dump stream debug info.
    fn dump(&self) -> String {
        String::new()
    }
}

/// Read
pub trait IStream<TW> {
    /// Try advance and panic in case of error.
    fn advance<'a>(&'a mut self, n: usize) -> TbitSlice<'a, TW> {
        let r = self.try_advance(n);
        assert!(r.is_ok());
        r.unwrap()
    }

    /// Try get n tbits from the stream, returning a slice to the buffer.
    fn try_advance<'a>(&'a mut self, n: usize) -> Fallible<TbitSlice<'a, TW>>;

    /// Commit advanced buffers from the internal sources.
    fn commit(&mut self);

    /// Dump stream debug info.
    fn dump(&self) -> String {
        String::new()
    }
}

impl<'b, TW> OStream<TW> for TbitSliceMut<'b, TW>
where
    TW: BasicTbitWord,
{
    fn try_advance<'a>(&'a mut self, n: usize) -> Fallible<TbitSliceMut<'a, TW>> {
        ensure!(n <= self.size(), "Output slice too short.");
        Ok(self.advance(n))
    }
    fn commit(&mut self) {}
    fn dump(&self) -> String {
        format!("{:?}", self)
    }
}

impl<'b, TW> IStream<TW> for TbitSlice<'b, TW>
where
    TW: BasicTbitWord,
{
    fn try_advance<'a>(&'a mut self, n: usize) -> Fallible<TbitSlice<'a, TW>> {
        ensure!(n <= self.size(), "Input slice too short.");
        Ok(self.advance(n))
    }
    fn commit(&mut self) {}
    fn dump(&self) -> String {
        format!("{:?}", self)
    }
}

pub struct NoOStream;

impl<TW> OStream<TW> for NoOStream {
    fn advance<'a>(&'a mut self, n: usize) -> TbitSliceMut<'a, TW> {
        assert!(false, "Advance can't be implemented for NoOStream");
        self.try_advance(n).unwrap()
    }
    fn try_advance<'a>(&'a mut self, _n: usize) -> Fallible<TbitSliceMut<'a, TW>> {
        bail!("Advance can't be implemented for NoOStream")
    }
    fn commit(&mut self) {}
}

pub struct NoIStream;

impl<TW> IStream<TW> for NoIStream {
    fn advance<'a>(&'a mut self, n: usize) -> TbitSlice<'a, TW> {
        assert!(false, "Advance can't be implemented for NoIStream");
        self.try_advance(n).unwrap()
    }
    fn try_advance<'a>(&'a mut self, _n: usize) -> Fallible<TbitSlice<'a, TW>> {
        bail!("Advance can't be implemented for NoIStream")
    }
    fn commit(&mut self) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use iota_streams_core::{
        sponge::{
            prp::PRP,
            spongos::Spongos,
        },
        tbits::{
            word::{
                SpongosTbitWord,
                StringTbitWord,
            },
            TbitSlice,
            TbitSliceMut,
            Tbits,
        },
    };
    use std::str::FromStr;

    fn wrap_absorb_tbits<TW, F, OS: OStream<TW>>(x: TbitSlice<TW>, s: &mut Spongos<TW, F>, os: &mut OS) -> ()
    where
        TW: SpongosTbitWord,
        F: PRP<TW>,
    {
        let n = x.size();
        let t = os.advance(n);
        x.copy(&t);
        s.absorb(x);
    }

    fn unwrap_absorb_tbits<TW, F, IS: IStream<TW>>(
        x: TbitSliceMut<TW>,
        s: &mut Spongos<TW, F>,
        is: &mut IS,
    ) -> Fallible<()>
    where
        TW: SpongosTbitWord,
        F: PRP<TW>,
    {
        let n = x.size();
        let t = is.try_advance(n)?;
        t.copy(&x);
        s.absorb(unsafe { x.as_const() });
        Ok(())
    }

    fn do_wrap_unwrap<TW, F>()
    where
        TW: SpongosTbitWord + StringTbitWord,
        F: PRP<TW> + Default,
    {
        let x = Tbits::<TW>::from_str("ABC").unwrap();
        let mut y = Tbits::<TW>::zero(x.size());

        let mut buf = Tbits::<TW>::zero(x.size());

        let tag = {
            let mut s = Spongos::<TW, F>::init();
            let mut b = buf.slice_mut();
            wrap_absorb_tbits(x.slice(), &mut s, &mut b);
            s.squeeze_tbits(81);
        };

        let tag2 = {
            let mut s = Spongos::<TW, F>::init();
            let mut b = buf.slice();
            let r = unwrap_absorb_tbits(y.slice_mut(), &mut s, &mut b);
            assert!(r.is_ok());
            s.squeeze_tbits(81);
        };

        assert_eq!(x, y);
        assert_eq!(tag, tag2);
    }

    #[test]
    pub fn wrap_unwrap() {
        use iota_streams_core::{
            sponge::prp::troika::Troika,
            tbits::trinary::Trit,
        };
        do_wrap_unwrap::<Trit, Troika>();
    }
}
