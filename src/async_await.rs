#[macro_export]
macro_rules! impl_sql {
    ( $sql_name:ident = $( { $kind:tt $name:ident ($($variant:tt $param:ident $ptype:tt)*) $doc:literal $s:tt $( $text:tt )+ } ),+ ) => {
        trait $sql_name {
            $( $crate::decl_method!{ $kind $name $doc () () () $($param $variant $ptype)* } )+
        }
        impl $sql_name for ::tokio_postgres::Client {
            $( $crate::impl_method!{ $kind $name () () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
        impl $sql_name for ::tokio_postgres::Transaction<'_> {
            $( $crate::impl_method!{ $kind $name () () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! decl_method {
    ( ? $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*, F>(&'st self $($fn_params)* , row_cb: F)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<(),::tokio_postgres::Error>> + Send + 'tr>>
        where
            F: FnMut(::tokio_postgres::Row) -> ::std::result::Result<(),::tokio_postgres::Error>,
            F: Send, F: 'tr, Self: 'tr, 'st: 'tr $(, $lt : 'tr)*;
    };
    ( ^ $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*>(&'st self $($fn_params)*)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::RowStream,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*;
    };
    ( % $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<'tr, 'st $(, $lt)* $($gen_type : ::tokio_postgres::types::ToSql)*, R>(&'st self $($fn_params)*) 
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::std::vec::Vec<R>,::tokio_postgres::Error>> + Send + 'tr>>
        where R: Send, R: ::std::convert::TryFrom<::tokio_postgres::Row>, ::tokio_postgres::Error: ::std::convert::From<R::Error>, Self: 'tr, 'st: 'tr $(, $lt : 'tr)*;
    };
    ( ! $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*>(&'st self $($fn_params)*)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<u64,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*;
    };
    ( -> $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*>(&'st self $($fn_params)*)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::Row,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*;
    };
    ( $kind:tt $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) $param:ident : _ $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($lt)*)
            ($($gen_type)*)
            ($($fn_params)* , $param : impl ::tokio_postgres::types::ToSql + Sync + Send + 'tr)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) $param:ident : ($plt:lifetime & $ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($lt)* $plt)
            ($($gen_type)*)
            ($($fn_params)* , $param : & $plt $ptype)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) $param:ident : ($ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($lt)*)
            ($($gen_type)*)
            ($($fn_params)* , $param : $ptype)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) $param:ident # [$alt:lifetime $gtype:ident] $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($lt)* $alt)
            ($($gen_type)* , $gtype : ::tokio_postgres::types::ToSql + Sync + Send + 'tr)
            ($($fn_params)* , $param : & $alt [ $gtype ] )
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) $param:ident # ($alt:lifetime $plt:lifetime & $ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($lt)* $alt $plt)
            ($($gen_type)*)
            ($($fn_params)* , $param : & $alt [ & $plt $ptype ] )
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) $param:ident # ($alt:lifetime $ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($lt)* $alt)
            ($($gen_type)*)
            ($($fn_params)* , $param : & $alt [ $ptype ] )
            $($tail)*
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_method {
    ( ? $name:ident () () () () => () $text:literal ) => {
        fn $name<'tr, 'st, F>(&'st self, mut row_cb: F)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<(),::tokio_postgres::Error>> + Send + 'tr>>
        where F: FnMut(::tokio_postgres::Row) -> ::std::result::Result<(),::tokio_postgres::Error>, F: Send, F: 'tr, Self: 'tr, 'st: 'tr
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let rows = self.query_raw( $text, [] as [&(dyn ::tokio_postgres::types::ToSql + Sync); 0] ).await?;
                ::futures_util::pin_mut!(rows);
                while let Some(row) = rows.try_next().await? {
                    row_cb(row)?;
                }
                Ok::<(),::tokio_postgres::Error>(())
            })
        }
    };
    ( ? $name:ident ($($lt:lifetime)*) () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)*, F>(&'st self $($fn_params)+ , mut row_cb: F)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<(),::tokio_postgres::Error>> + Send + 'tr>>
        where F: FnMut(::tokio_postgres::Row) -> ::std::result::Result<(),::tokio_postgres::Error>, F: Send, F: 'tr, Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let rows = self.query_raw(
                    $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                    [& $head as &(dyn ::tokio_postgres::types::ToSql + Sync) $(, & $tail)* ]
                ).await?;
                ::futures_util::pin_mut!(rows);
                while let Some(row) = rows.try_next().await? {
                    row_cb(row)?;
                }
                Ok::<(),::tokio_postgres::Error>(())
            })
        }
    };
    ( ? $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*, F>(&'st self $($fn_params)+, mut row_cb: F)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<(),::tokio_postgres::Error>> + Send + 'tr>>
        where F: FnMut(::tokio_postgres::Row) -> ::std::result::Result<(),::tokio_postgres::Error>, F: Send, F: 'tr, Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
                let mut args = ::std::vec::Vec::<&(dyn ::tokio_postgres::types::ToSql + Sync)>::with_capacity($crate::num_args!($($pv $param)+));
                let mut i = 0;
                $crate::dynamic_sql!(stmt args i $($text)+);
                let rows = self.query_raw(&stmt, args).await?;
                ::futures_util::pin_mut!(rows);
                while let Some(row) = rows.try_next().await? {
                    row_cb(row)?;
                }
                Ok::<(),::tokio_postgres::Error>(())
            })
        }
    };
    ( ^ $name:ident () () () () => () $text:literal ) => {
        fn $name<'tr, 'st>(&'st self)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::RowStream,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(
                self.query_raw( $text, [] as [&(dyn ::tokio_postgres::types::ToSql + Sync); 0] )
            )
        }
    };
    ( ^ $name:ident ($($lt:lifetime)*) () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)*>(&'st self $($fn_params)+)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::RowStream,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                self.query_raw(
                    $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                    [& $head as &(dyn ::tokio_postgres::types::ToSql + Sync) $(, & $tail)* ]
                ).await
            })
        }
    };
    ( ^ $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*>(&'st self $($fn_params)+)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::RowStream,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
                let mut args = ::std::vec::Vec::<&(dyn ::tokio_postgres::types::ToSql + Sync)>::with_capacity($crate::num_args!($($pv $param)+));
                let mut i = 0;
                $crate::dynamic_sql!(stmt args i $($text)+);
                self.query_raw(&stmt, args).await
            })
        }
    };
    ( % $name:ident () () () () => () $text:literal ) => {
        fn $name<'tr, 'st, R>(&'st self)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::std::vec::Vec<R>,::tokio_postgres::Error>> + Send + 'tr>>
        where R: Send, R: ::std::convert::TryFrom<::tokio_postgres::Row>, ::tokio_postgres::Error: ::std::convert::From<R::Error>, Self: 'tr, 'st: 'tr
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let rows = self.query_raw( $text, [] as [&(dyn ::tokio_postgres::types::ToSql + Sync); 0] ).await?;
                ::futures_util::pin_mut!(rows);
                let mut data = ::std::vec::Vec::new();
                while let Some(row) = rows.try_next().await? {
                    let item = R::try_from(row)?;
                    data.push(item);
                }
                Ok(data)
            })
        }
    };
    ( % $name:ident ($($lt:lifetime)*) () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)*, R>(&'st self $($fn_params)+)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::std::vec::Vec<R>,::tokio_postgres::Error>> + Send + 'tr>>
        where R: Send, R: ::std::convert::TryFrom<::tokio_postgres::Row>, ::tokio_postgres::Error: ::std::convert::From<R::Error>, Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let rows = self.query_raw(
                    $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                    [& $head as &(dyn ::tokio_postgres::types::ToSql + Sync) $(, & $tail)* ]
                ).await?;
                ::futures_util::pin_mut!(rows);
                let mut data = ::std::vec::Vec::new();
                while let Some(row) = rows.try_next().await? {
                    let item = R::try_from(row)?;
                    data.push(item);
                }
                Ok(data)
            })
        }
    };
    ( % $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*, R>(&'st self $($fn_params)+)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::std::vec::Vec<R>,::tokio_postgres::Error>> + Send + 'tr>>
        where R: Send, R: ::std::convert::TryFrom<::tokio_postgres::Row>, ::tokio_postgres::Error: ::std::convert::From<R::Error>, Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            use ::futures_util::TryStreamExt;

            ::std::boxed::Box::pin(async move {
                let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
                let mut args = ::std::vec::Vec::<&(dyn ::tokio_postgres::types::ToSql + Sync)>::with_capacity($crate::num_args!($($pv $param)+));
                let mut i = 0;
                $crate::dynamic_sql!(stmt args i $($text)+);
                let rows = self.query_raw(&stmt, args).await?;
                ::futures_util::pin_mut!(rows);
                let mut data = ::std::vec::Vec::new();
                while let Some(row) = rows.try_next().await? {
                    let item = R::try_from(row)?;
                    data.push(item);
                }
                Ok(data)
            })
        }
    };
    ( ! $name:ident () () () () => () $text:literal ) => {
        fn $name<'tr, 'st>(&'st self)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<u64,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr
        {
            ::std::boxed::Box::pin(async move {
                self.execute( $text, &[] ).await
            })
        }
    };
    ( ! $name:ident ($($lt:lifetime)*) () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)*>(&'st self $($fn_params)+ )
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<u64,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            ::std::boxed::Box::pin(async move {
                self.execute(
                    $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                    &[& $head as &(dyn ::tokio_postgres::types::ToSql + Sync) $(, & $tail)* ]
                ).await
            })
        }
    };
    ( ! $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*>(&'st self $($fn_params)+ )
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<u64,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            ::std::boxed::Box::pin(async move {
                let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
                let mut args = ::std::vec::Vec::<&(dyn ::tokio_postgres::types::ToSql + Sync)>::with_capacity($crate::num_args!($($pv $param)+));
                let mut i = 0;
                $crate::dynamic_sql!(stmt args i $($text)+);
                self.execute_raw(&stmt, args).await
            })
        }
    };
    ( -> $name:ident () () () () => () $text:literal ) => {
        fn $name<'tr, 'st>(&'st self)
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::Row,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr
        {
            ::std::boxed::Box::pin(async move {
                self.query_one( $text, &[] ).await
            })
        }
    };
    ( -> $name:ident ($($lt:lifetime)*) () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)*>(&'st self $($fn_params)+ )
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::Row,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            ::std::boxed::Box::pin(async move {
                self.query_one(
                    $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                    &[& $head as &(dyn ::tokio_postgres::types::ToSql + Sync) $(, & $tail)* ]
                ).await
            })
        }
    };
    ( -> $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<'tr, 'st $(, $lt)* $($gen_type)*>(&'st self $($fn_params)+ )
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::std::result::Result<::tokio_postgres::Row,::tokio_postgres::Error>> + Send + 'tr>>
        where Self: 'tr, 'st: 'tr $(, $lt : 'tr)*
        {
            ::std::boxed::Box::pin(async move {
                let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
                let mut args = ::std::vec::Vec::<&(dyn ::tokio_postgres::types::ToSql + Sync)>::with_capacity($crate::num_args!($($pv $param)+));
                let mut i = 0;
                $crate::dynamic_sql!(stmt args i $($text)+);
                self.query_one(&stmt, &args).await
            })
        }
    };
    ( $kind:tt $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ($param:ident : _ $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($lt)*)
            ($($gen_type)*)
            ($($fn_params)* , $param : impl ::tokio_postgres::types::ToSql + Sync + Send + 'tr)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ($param:ident : ($plt:lifetime & $ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($lt)* $plt)
            ($($gen_type)*)
            ($($fn_params)* , $param : & $plt $ptype)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ($param:ident : ($ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($lt)*)
            ($($gen_type)*)
            ($($fn_params)* , $param : $ptype)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ($param:ident # [$alt:lifetime $gtype:ident] $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($lt)* $alt)
            ($($gen_type)*  , $gtype : ::tokio_postgres::types::ToSql + Sync + Send + 'tr)
            ($($fn_params)* , $param : & $alt [ $gtype ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ($param:ident # ($alt:lifetime $plt:lifetime & $ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($lt)* $alt $plt)
            ($($gen_type)*)
            ($($fn_params)* , $param : & $alt [ & $plt $ptype ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($lt:lifetime)*) ($($gen_type:tt)*) ($($fn_params:tt)*) ($param:ident # ($alt:lifetime $ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($lt)* $alt)
            ($($gen_type)*)
            ($($fn_params)* , $param : & $alt [ $ptype ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
}
