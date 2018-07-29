#![doc(hidden)]

use frunk::hlist::{HList, HCons};

use hlist_extension::{Visitor, Gatherer};
use rpc::system::RPCService;

impl<'me, X, Tail> Visitor<'me, RPCService> for HCons<X, Tail>
where
    X: 'me + RPCService,
    Tail: Visitor<'me, RPCService>,
{
    fn visit<F>(&'me self, closure: F)
    where
        F: Fn(&'me RPCService) -> (),
    {
        let closure_argument: &RPCService = &self.head;
        (closure)(closure_argument);
        self.tail.visit(closure);
    }
}

impl<'me, X, Tail, Result> Gatherer<'me, RPCService, Result> for HCons<X, Tail>
where
    X: 'me + RPCService + Sized,
    Tail: Gatherer<'me, RPCService, Result> + Sized,
    Result: 'me,
    Self: HList,
{
    /*
    fn gather<F>(&'me self, collector: &'me mut [Option<Result>; Self::LEN], closure: F)
    where
        F: for<'collect> Fn(&'me RPCService, &'collect mut Option<Result>) -> (),
    {
        unsafe {
            self.internal_gather(self, collector);
        }
    }
    */

    unsafe fn gather<F>(&'me self, collector: &'me mut [Option<Result>], closure: F)
    where
        F: for<'collect> Fn(&'me RPCService, &'collect mut Option<Result>) -> (),
    {
        {
            let closure_argument: &RPCService = &self.head;
            let element = collector.get_mut(Self::LEN - 1).unwrap();
            (closure)(closure_argument, element);
        }
        self.tail.gather(collector, closure);
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::cell::Cell;
    use std::default::Default;

    #[derive(Debug, Default)]
    struct SOne;
    impl RPCService for SOne {}

    #[derive(Debug, Default)]
    struct STwo;
    impl RPCService for STwo {}

    #[derive(Debug, Default)]
    struct SThree;
    impl RPCService for SThree {}

    #[test]
    fn frunk_gather() {
        let counter = Cell::new(0u32);

        let example: Hlist!(SOne, STwo, SThree) = Default::default();
        let mut collector = [None; <Hlist!(SOne, STwo, SThree)>::LEN];

        unsafe {
            Gatherer::<RPCService, u32>::gather(&example, &mut collector, move |service, out| {
                let value = counter.get();
                counter.set(value + 1);
                *out = Some(value);
            });
        }

        assert_eq!(collector, [Some(2), Some(1), Some(0)]);
    }
}
