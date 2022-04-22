-- name: new_genre->
-- Creates new genre
-- # Parameters
-- param: name: &str - genre name
INSERT INTO genre (genre_id, name)
SELECT Coalesce(Max(genre_id),0) + 1, :name
  FROM genre
RETURNING genre_id;

-- name: delete_genre
-- Deletes genre
-- # Parameters
-- param: id: i32 - genre ID
DELETE FROM genre WHERE genre_id = :id;
