use paste::paste;

use async_trait::async_trait;

macro_rules! declare_async_callable {
    ($I:literal, ($($ARG:literal),*)) => {
        declare_async_callable!([Once], $I, ($($ARG),*));
        declare_async_callable!([Mut], $I, ($($ARG),*));
        declare_async_callable!([], $I, ($($ARG),*));
    };

    ([$($FNTYPE:ident)?], $I:literal, ($($ARG:literal),*)) => {
        paste! {
            #[async_trait]
            pub trait [<AsyncCallable $($FNTYPE)? $I>]<'a, $([<Arg $ARG>]),*, R> {
                async fn call(&'a self, $([<arg $ARG>] : [<Arg $ARG>]),*) -> R;
            }

            #[async_trait]
            impl <
                'a,
                $([<Arg $ARG>] : Send + Sync + 'a),*,
                R,
                T: Sync
                    + async_fn_traits::[<AsyncFn $I>] <$([<Arg $ARG>]),*, Output = R>
            >
                [<AsyncCallable $($FNTYPE)? $I>] <'a, $([<Arg $ARG>]),*, R> for T
            where
                <T as async_fn_traits::[<AsyncFn $I>]<$([<Arg $ARG>]),*>>::OutputFuture: Send + Sync
            {
                async fn call(&'a self, $([<arg $ARG>] : [<Arg $ARG>]),*) -> R {
                    self($([<arg $ARG>]),*).await
                }
            }
        }
    };
}

declare_async_callable!(1, (0));
declare_async_callable!(2, (0, 1));
declare_async_callable!(3, (0, 1, 2));
declare_async_callable!(4, (0, 1, 2, 3));
declare_async_callable!(5, (0, 1, 2, 3, 4));
declare_async_callable!(6, (0, 1, 2, 3, 4, 5));
declare_async_callable!(7, (0, 1, 2, 3, 4, 5, 6));
declare_async_callable!(8, (0, 1, 2, 3, 4, 5, 6, 7));
declare_async_callable!(9, (0, 1, 2, 3, 4, 5, 6, 7, 8));
declare_async_callable!(10, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9));
declare_async_callable!(11, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10));
declare_async_callable!(12, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11));
declare_async_callable!(13, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12));
declare_async_callable!(14, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13));
declare_async_callable!(15, (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14));
