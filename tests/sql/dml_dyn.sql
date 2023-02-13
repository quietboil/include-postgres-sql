-- name: delete_genres
-- Deletes generes by name
-- # Parameters
-- param: names: &str - list of genre names
DELETE FROM genre WHERE name IN (:names)
