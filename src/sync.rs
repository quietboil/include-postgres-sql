/**
Generates Rust code to use included SQL.

This macro defines a trait with methods to access data and implements it for `postgres::Client` and `postgres::Transaction`.

This macro recognizes and generates 5 variants of database access methods using the following selectors:
* `?` - methods that process rows retrieved by `SELECT`,
* `^` - methods that return raw rows retrieved by `SELECT`,
* `%` - methods that return vector of structs (a struct per returned row)
* `!` - methods that execute all other non-`SELECT` methods, and
* `->` - methods that execute `RETURNING` statements and provide access to returned data.

For `SELECT` statements (`?`) like:

```sql
-- name: get_loaned_books?
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id
```

The method with the following signature is generated:

```rust , ignore
fn get_loaned_books<F>(&self, user_id: &str, row_callback: F) -> Result<(),postgres::Error>
where F: FnMut(postgres::Row) -> Result<(),postgres::Error>;
```

For `SELECT` statements (`^`):

```sql
-- name: get_loaned_books^
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id
```

The method with the following signature is generated:

```rust , ignore
fn get_loaned_books<'a>(&'a self, user_id: &str) -> Result<postgres::RowIter<'a>,postgres::Error>;
```

For `SELECT` statements (`%`):

```sql
-- name: get_loaned_books%
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id
```

The method with the following signature is generated:

```rust , ignore
fn get_loaned_books<R>(&self, user_id: &str) -> Result<Vec<R>,postgres::Error>
where R: TryFrom<postres::Row>, postgres::Error: From<R::Error>;
```

For non-select statements (`!`) - INSERT, UPDATE, DELETE, etc. - like:

```sql
-- name: loan_books!
-- param: user_id: &str
-- param: book_ids: i32
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids)
```

The method with the following signature is generated:

```rust , ignore
fn loan_books(&self, user_id: &str, book_ids: &[i32]) -> Result<u64,postgres::Error>;
```

For DELETE, INSERT, and UPDATE statements that return data via `RETURNING` clause (`->`) like:

```sql
-- name: add_new_book->
-- param: isbn: &str
-- param: book_title: &str
INSERT INTO library (isbn, book_title)
VALUES (:isbn, :book_title)
RETURNING book_id
```

The method with the following signature is generated:

```rust , ignore
fn add_new_book(&self, isbn: &str, book_title: &str) -> Result<postgres::Row,postgres::Error>;
```

### Tokio-Postgres

**Note** that when **include-postgres-sql** is used with the `tokio` feature, the generated methods will be `async`.
*/
#[macro_export]
macro_rules! impl_sql {
    ( $sql_name:ident = $( { $kind:tt $name:ident ($($variant:tt $param:ident $ptype:tt)*) $doc:literal $s:tt $( $text:tt )+ } ),+ ) => {
        trait $sql_name {
            $( $crate::decl_method!{ $kind $name $doc () () $($param $variant $ptype)* } )+
        }
        impl $sql_name for ::postgres::Client {
            $( $crate::impl_method!{ $kind $name () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
        impl $sql_name for ::postgres::Transaction<'_> {
            $( $crate::impl_method!{ $kind $name () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! decl_method {
    ( ? $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : ::postgres::types::ToSql ,)* F>(&mut self $($fn_params)* , row_cb: F) -> ::std::result::Result<(),::postgres::Error>
        where F: FnMut(::postgres::Row) -> ::std::result::Result<(),::postgres::Error>;
    };
    ( ^ $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<'a $(, $gen_type : ::postgres::types::ToSql)*>(&'a mut self $($fn_params)*) -> ::std::result::Result<::postgres::RowIter<'a>,::postgres::Error>;
    };
    ( % $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : ::postgres::types::ToSql ,)* R>(&mut self $($fn_params)*) -> ::std::result::Result<::std::vec::Vec<R>,::postgres::Error>
        where R: ::std::convert::TryFrom<::postgres::Row>, ::postgres::Error: ::std::convert::From<R::Error>;
    };
    ( ! $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : ::postgres::types::ToSql),*>(&mut self $($fn_params)*) -> ::std::result::Result<u64,::postgres::Error>;
    };
    ( -> $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : ::postgres::types::ToSql),*>(&mut self $($fn_params)*) -> ::std::result::Result<::postgres::Row,::postgres::Error>;
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident : _ $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : impl ::postgres::types::ToSql + Sync)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident : ($ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : $ptype)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident # [$gtype:ident] $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)* $gtype)
            ($($fn_params)* , $param : & [ $gtype ] )
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident # ($ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : & [ $ptype ] )
            $($tail)*
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_method {
    ( ? $name:ident () () () => () $text:literal ) => {
        fn $name<F>(&mut self, mut row_cb: F) -> ::std::result::Result<(),::postgres::Error>
        where F: FnMut(::postgres::Row) -> ::std::result::Result<(),::postgres::Error>
        {
            use ::postgres::fallible_iterator::FallibleIterator;

            let mut rows = self.query_raw( $text, [] as [&dyn ::postgres::types::ToSql; 0] )?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ? $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<F>(&mut self $($fn_params)+ , mut row_cb: F) -> ::std::result::Result<(),::postgres::Error>
        where F: FnMut(::postgres::Row) -> ::std::result::Result<(),::postgres::Error>
        {
            use ::postgres::fallible_iterator::FallibleIterator;

            let mut rows = self.query_raw(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                [& $head as &(dyn ::postgres::types::ToSql + Sync) $(, & $tail)* ]
            )?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ? $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : ::postgres::types::ToSql ,)* F>(&mut self $($fn_params)+, mut row_cb: F) -> ::std::result::Result<(),::postgres::Error>
        where F: FnMut(::postgres::Row) -> ::std::result::Result<(),::postgres::Error>
        {
            use ::postgres::fallible_iterator::FallibleIterator;

            let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
            let mut args = ::std::vec::Vec::<&dyn ::postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            let mut rows = self.query_raw(&stmt, args)?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ^ $name:ident () () () => () $text:literal ) => {
        fn $name<'a>(&'a mut self) -> ::std::result::Result<::postgres::RowIter<'a>,::postgres::Error> {
            self.query_raw( $text, [] as [&dyn ::postgres::types::ToSql; 0] )
        }
    };
    ( ^ $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<'a>(&'a mut self $($fn_params)+) -> ::std::result::Result<::postgres::RowIter<'a>,::postgres::Error> {
            self.query_raw(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                [& $head as &(dyn ::postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( ^ $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<'a $(, $gen_type : ::postgres::types::ToSql)*>(&'a mut self $($fn_params)+) -> ::std::result::Result<::postgres::RowIter<'a>,::postgres::Error> {
            let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
            let mut args = ::std::vec::Vec::<&dyn ::postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            self.query_raw(&stmt, args)
        }
    };
    ( % $name:ident () () () => () $text:literal ) => {
        fn $name<R>(&mut self) -> ::std::result::Result<::std::vec::Vec<R>,::postgres::Error>
        where R: ::std::convert::TryFrom<::postgres::Row>, ::postgres::Error: ::std::convert::From<R::Error>
        {
            use ::postgres::fallible_iterator::FallibleIterator;
            let mut data = ::std::vec::Vec::new();
            let mut rows = self.query_raw( $text, [] as [&dyn ::postgres::types::ToSql; 0] )?;
            while let Some(row) = rows.next()? {
                let item = R::try_from(row)?;
                data.push(item);
            }
            Ok(data)
        }
    };
    ( % $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<R>(&mut self $($fn_params)+) -> ::std::result::Result<::std::vec::Vec<R>,::postgres::Error>
        where R: ::std::convert::TryFrom<::postgres::Row>, ::postgres::Error: ::std::convert::From<R::Error>
        {
            use ::postgres::fallible_iterator::FallibleIterator;
            let mut data = ::std::vec::Vec::new();
            let mut rows = self.query_raw(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                [& $head as &(dyn ::postgres::types::ToSql + Sync) $(, & $tail)* ]
            )?;
            while let Some(row) = rows.next()? {
                let item = R::try_from(row)?;
                data.push(item);
            }
            Ok(data)
        }
    };
    ( % $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : ::postgres::types::ToSql ,)* R>(&mut self $($fn_params)+) -> ::std::result::Result<::std::vec::Vec<R>,::postgres::Error>
        where R: ::std::convert::TryFrom<::postgres::Row>, ::postgres::Error: ::std::convert::From<R::Error>
        {
            use ::postgres::fallible_iterator::FallibleIterator;
            let mut data = ::std::vec::Vec::new();
            let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
            let mut args = ::std::vec::Vec::<&dyn ::postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            let mut rows = self.query_raw(&stmt, args)?;
            while let Some(row) = rows.next()? {
                let item = R::try_from(row)?;
                data.push(item);
            }
            Ok(data)
        }
    };
    ( ! $name:ident () () () => () $text:literal ) => {
        fn $name(&mut self) -> ::std::result::Result<u64,::postgres::Error> {
            self.execute( $text, &[] )
        }
    };
    ( ! $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name(&mut self $($fn_params)+ ) -> ::std::result::Result<u64,::postgres::Error> {
            self.execute(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                &[& $head as &(dyn ::postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( ! $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : ::postgres::types::ToSql),*>(&mut self $($fn_params)+ ) -> ::std::result::Result<u64,::postgres::Error> {
            let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
            let mut args = ::std::vec::Vec::<&(dyn ::postgres::types::ToSql + Sync)>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            self.execute(&stmt, args.as_slice())
        }
    };
    ( -> $name:ident () () () => () $text:literal ) => {
        fn $name(&mut self) -> ::std::result::Result<::postgres::Row,::postgres::Error> {
            self.query_one( $text, &[] )
        }
    };
    ( -> $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name(&mut self $($fn_params)+ ) -> ::std::result::Result<::postgres::Row,::postgres::Error> {
            self.query_one(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                &[& $head as &(dyn ::postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( -> $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : ::postgres::types::ToSql),*>(&mut self $($fn_params)+ ) -> ::std::result::Result<::postgres::Row,::postgres::Error> {
            let mut stmt = ::std::string::String::with_capacity($crate::sql_len!($($text)+));
            let mut args = ::std::vec::Vec::<&dyn ::postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            self.query_one(&stmt, &args)
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident : _ $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)*)
            ($($fn_params)* , $param : impl ::postgres::types::ToSql + Sync)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident : ($ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)*)
            ($($fn_params)* , $param : $ptype)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident # [$gtype:ident] $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)* $gtype)
            ($($fn_params)* , $param : & [ $gtype ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident # ($ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)*)
            ($($fn_params)* , $param : & [ $ptype ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
}
