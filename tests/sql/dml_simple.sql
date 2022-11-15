-- name: new_genre
-- Creates new genre
-- # Parameters
-- param: id: i32 - new genre ID
-- param: name: &str - genre name
INSERT INTO genre (genre_id, name) VALUES (:id, :name);

-- name: delete_genre
-- Deletes genre
-- # Parameters
-- param: id: i32 - genre ID
DELETE FROM genre WHERE genre_id = :id;
