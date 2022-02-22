#![cfg_attr(docsrs, doc = include_str!("../docs/index.md"))]

pub use include_sql::include_sql;

/**
Generates Rust code to use included SQL.

This macro defines a trait with methods to access data and implements it for `postgres::Client`.

This macro recognizes and generates 3 variants of database access methods using the following selectors:
* `?` - methods that process rows retrieved by `SELECT`,
* `!` - methods that execute all other non-`SELECT` methods, and
* `->` - methods that execute `RETURNING` statements and provide access to returned data.

For `SELECT` statements (`?`) like:

```sql
-- name: get_loaned_books?
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id;
```

The method with the following signature is generated:

```rust , ignore
fn get_loaned_books<F>(
    &self,
    user_id: &str,
    row_callback: F
) -> Result<(),postgres::Error>
where F: Fn(postgres::Row) -> Result<(),postgres::Error>;
```

For non-select statements (`!`) - INSERT, UPDATE, DELETE, etc. - like:

```sql
-- name: loan_books!
-- param: user_id: &str
-- param: book_ids: i32
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids);
```

The method with the following signature is generated:

```rust , ignore
fn loan_books(
    &self,
    user_id: &str,
    book_ids: &[i32]
) -> Result<u64,postgres::Error>;
```

For DELETE, INSERT, and UPDATE statements that return data via `RETURNING` clause (`->`) like:

```sql
-- name: add_new_book->
-- param: isbn: &str
-- param: book_title: &str
INSERT INTO library (isbn, book_title)
VALUES (:isbn, :book_title)
RETURNING book_id;
```

The method with the following signature is generated:

```rust , ignore
fn add_new_book(
    &self,
    isbn: &str,
    book_title: &str
) -> Result<postgres::Row,postgres::Error>;
```
*/
#[macro_export]
macro_rules! impl_sql {
    ( $sql_name:ident = $( { $kind:tt $name:ident ($($variant:tt $param:ident $ptype:tt)*) $doc:literal $s:tt $( $text:tt )+ } ),+ ) => {
        trait $sql_name {
            $( $crate::decl_method!{ $kind $name $doc () () $($param $variant $ptype)* } )+
        }
        impl $sql_name for postgres::Client {
            $( $crate::impl_method!{ $kind $name () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
        impl $sql_name for postgres::Transaction<'_> {
            $( $crate::impl_method!{ $kind $name () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! decl_method {
    ( ? $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : postgres::types::ToSql ,)* F>(&mut self $($fn_params)* , row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>;
    };
    ( ! $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)*) -> std::result::Result<u64,postgres::Error>;
    };
    ( -> $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)*) -> std::result::Result<postgres::Row,postgres::Error>;
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident : _ $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : impl postgres::types::ToSql + Sync)
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
        fn $name<F>(&mut self, row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>
        {
            use postgres::fallible_iterator::FallibleIterator;

            let mut rows = self.query_raw( $text, [] as [&dyn postgres::types::ToSql; 0] )?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ? $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<F>(&mut self $($fn_params)+ , row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>
        {
            use postgres::fallible_iterator::FallibleIterator;

            let mut rows = self.query_raw(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                [& $head as &(dyn postgres::types::ToSql + Sync) $(, & $tail)* ]
            )?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ? $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : postgres::types::ToSql ,)* F>(&mut self $($fn_params)+, row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>
        {
            use postgres::fallible_iterator::FallibleIterator;

            let mut stmt = String::with_capacity($crate::sql_len!($($text)+));
            let mut args = Vec::<&dyn postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            let mut rows = self.query_raw(&stmt, args)?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ! $name:ident () () () => () $text:literal ) => {
        fn $name(&mut self) -> std::result::Result<u64,postgres::Error> {
            self.execute( $text, &[] )
        }
    };
    ( ! $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name(&mut self $($fn_params)+ ) -> std::result::Result<u64,postgres::Error> {
            self.execute(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                &[& $head as &(dyn postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( ! $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)+ ) -> std::result::Result<u64,postgres::Error> {
            let mut stmt = String::with_capacity($crate::sql_len!($($text)+));
            let mut args = Vec::<&dyn postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            self.execute(&stmt, &args)
        }
    };
    ( -> $name:ident () () () => () $text:literal ) => {
        fn $name(&mut self) -> std::result::Result<postgres::Row,postgres::Error> {
            self.query_one( $text, &[] )
        }
    };
    ( -> $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name(&mut self $($fn_params)+ ) -> std::result::Result<postgres::Row,postgres::Error> {
            self.query_one(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                &[& $head as &(dyn postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( -> $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)+ ) -> std::result::Result<postgres::Row,postgres::Error> {
            let mut stmt = String::with_capacity($crate::sql_len!($($text)+));
            let mut args = Vec::<&dyn postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
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
            ($($fn_params)* , $param : impl postgres::types::ToSql + Sync)
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

#[macro_export]
#[doc(hidden)]
macro_rules! sql_literal {
    ($($name:ident)+ => $text:literal) => {
        $text
    };
    ($($name:ident)+ => $text:literal : $param:ident) => {
        std::concat!( $text, '$', include_sql::index_of!($param in [ $( $name ),+ ] + 1) )
    };
    ($($name:ident)+ => $text:literal : $param:ident $($tail:tt)+) => {
        std::concat!(
            $text, '$', include_sql::index_of!($param in [ $( $name ),+ ] + 1),
            $crate::sql_literal!($($name)+ => $($tail)+)
        )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! num_args {
    () => { 0 };
    (: $head:ident $($tail:tt)*) => { 1 + $crate::num_args!($($tail)*) };
    (# $head:ident $($tail:tt)*) => { $head.len() + $crate::num_args!($($tail)*) };
}

#[macro_export]
#[doc(hidden)]
macro_rules! sql_len {
    () => { 0 };
    ($text:literal $($tail:tt)*) => { $text.len() + $crate::sql_len!($($tail)*) };
    (: $head:ident $($tail:tt)*) => { 3 + $crate::sql_len!($($tail)*) };
    (# $head:ident $($tail:tt)*) => { $head.len() * 5 + $crate::sql_len!($($tail)*) };
}

#[macro_export]
#[doc(hidden)]
macro_rules! dynamic_sql {
    ($stmt:ident $args:ident $i:ident) => {};
    ($stmt:ident $args:ident $i:ident $text:literal $($tail:tt)*) => {
        $stmt.push_str($text);
        $crate::dynamic_sql!($stmt $args $i $($tail)*);
    };
    ($stmt:ident $args:ident $i:ident : $param:ident $($tail:tt)*) => {
        $i += 1;
        $stmt.push_str(&format!("${}", $i));
        $args.push(&$param);
        $crate::dynamic_sql!($stmt $args $i $($tail)*);
    };
    ($stmt:ident $args:ident $i:ident # $param:ident $($tail:tt)*) => {
        let mut iter = $param.into_iter();
        if let Some(arg) = iter.next() {
            $i += 1;
            $stmt.push_str(&format!("${}", $i));
            $args.push(arg);
            while let Some(arg) = iter.next() {
                $i += 1;
                $stmt.push_str(&format!(", ${}", $i));
                $args.push(arg);
            }
        } else {
            $stmt.push_str("NULL");
        }
        $crate::dynamic_sql!($stmt $args $i $($tail)*);
    };
}
