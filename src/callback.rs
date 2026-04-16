use std::future::Future;

pub(crate) trait CallResponse {
    const ID: u32;
    fn parse(bytes: &[u8]) -> Self;
}

pub(crate) trait Handler<Args>: Clone + 'static {
    type Output;
    type Future: Future<Output = Self::Output>;

    fn handler_call(&self, args: Args) -> Self::Future;
}

mod sealed {
    pub trait TupleInsert<T> {
        type FrontInsert;
        type BackInsert;
        fn front_insert(self, insert: T) -> Self::FrontInsert;
        fn back_insert(self, insert: T) -> Self::BackInsert;
    }
}

macro_rules! factory_tuple ({ $($param:ident)* } => {
    impl<Func, Fut, $($param,)*> Handler<($($param,)*)> for Func
    where
        Func: Fn($($param),*) -> Fut + Clone + 'static,
        Fut: Future,
    {
        type Output = Fut::Output;
        type Future = Fut;

        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[inline]
        fn handler_call(&self, ($($param,)*): ($($param,)*)) -> Self::Future {
            (self)($($param,)*)
        }
    }

    impl<InsertType, $($param,)*> sealed::TupleInsert<InsertType> for ($($param,)*) {
        type FrontInsert = (InsertType, $($param,)*);
        type BackInsert = ($($param,)* InsertType,);

        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[inline]
        fn front_insert(self, insert: InsertType) -> Self::FrontInsert {
            let ($($param,)*) = self;
            (insert, $($param,)*)
        }

        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[inline]
        fn back_insert(self, insert: InsertType) -> Self::BackInsert {
            let ($($param,)*) = self;
            ($($param,)* insert,)
        }
    }
});

factory_tuple! { }
factory_tuple! { A }
factory_tuple! { A B }
factory_tuple! { A B C }
factory_tuple! { A B C D }
factory_tuple! { A B C D E }
factory_tuple! { A B C D E F }
factory_tuple! { A B C D E F G }
factory_tuple! { A B C D E F G H }
factory_tuple! { A B C D E F G H I }
factory_tuple! { A B C D E F G H I J }
factory_tuple! { A B C D E F G H I J K }
factory_tuple! { A B C D E F G H I J K L }
factory_tuple! { A B C D E F G H I J K L M }
factory_tuple! { A B C D E F G H I J K L M N }
factory_tuple! { A B C D E F G H I J K L M N O }
factory_tuple! { A B C D E F G H I J K L M N O P }

#[must_use = "CallResult isn't used anywhere"]
pub struct CallResult<M : CallResponse> {
    receiver: std::sync::mpsc::Receiver<M>,
    context: crate::Context
}

#[must_use = "CallResultAsync isn't used anywhere"]
pub struct CallResultAsync<M : CallResponse, T: Clone> {
    vars: T,
    inner: CallResult<M>
}

pub enum CallResultError {
    Timeout,
    Terminated
}

#[allow(unused)]
impl<M> CallResult<M>
where
    M : CallResponse {
    
    pub fn add<J>(self, var: J) -> CallResultAsync<M, (J,)>
    where
        J: Clone
    {
        CallResultAsync { 
            inner: self,
            vars: (var,)
        }
    }

    pub fn with<F>(self, func: F) -> impl Fn(M) -> Box<dyn Future<Output = ()>>
    where
        F: Handler<(crate::Context, M,), Output = ()>,
        F::Future: Send + 'static {
        move |response: M| {
            Box::new(func.handler_call((self.context.clone(), response)))
        }
    }

    pub fn wait(self, timeout: std::time::Duration) -> Result<M, CallResultError> {
        match self.receiver.recv_timeout(timeout) {
            Ok(result) => Ok(result),
            Err(err) => match err {
                std::sync::mpsc::RecvTimeoutError::Timeout => Err(CallResultError::Timeout),
                std::sync::mpsc::RecvTimeoutError::Disconnected => Err(CallResultError::Terminated)
            }
        }
    }
}

impl crate::Context {
    pub(crate) fn proccess_call_result<M>(&self, id: u64) -> CallResult<M>
    where
        M: CallResponse {

        let mut context = self.clone();

        let (sender, receiver) = std::sync::mpsc::channel::<M>();

        let dispatch = move |bytes: &[u8]| {
            sender.send(M::parse(bytes))
        };

        CallResult::new(receiver, context)
    }
}

impl<M> CallResult<M>
where
    M: CallResponse {

    pub(crate) fn new(receiver: std::sync::mpsc::Receiver<M>, context: crate::Context) -> Self {
        Self { receiver, context }
    }
}

/*
#[allow(unused)]
impl<M, T> Register<M, T>
where
    M: SteamCallback + Default,
    T: Clone,
    //steam::proto::EMessage: EMessageUnwrap<M> {
    
    pub fn add<J>(self, var: J) -> Register<M, <T as sealed::TupleInsert<J>>::BackInsert>
    where
        T: sealed::TupleInsert<J>,
        T::BackInsert: Clone
    {
        Register { 
            vars: self.vars.back_insert(var),
            phantom: PhantomData,
        }
    }

    pub fn handler<F>(self, func: F) -> impl Fn(crate::Context, /*steam::proto::EMessage*/) -> Box<dyn Future<Output = ()>>
    where
        T: sealed::TupleInsert</*steam::proto::Message<M>*/>,
        T::FrontInsert: sealed::TupleInsert<crate::Context>,
        F: Handler<<T::FrontInsert as sealed::TupleInsert<crate::Context>>::FrontInsert, Output = ()>,
        F::Future: Send + 'static {
        move |context: crate::Context, callback: steam::proto::EMessage| {
            Box::new(func.handler_call( sealed::TupleInsert::front_insert(self.vars.clone().front_insert(message.unwrap()), context)))
        }
    }
}
*/