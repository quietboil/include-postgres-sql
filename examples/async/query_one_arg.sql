-- name: get_artist_albums?
-- Retrieves albums of the specified artist
-- # Parameters
-- param: artist_name: &str - Artist name
SELECT artist.name AS artist_name, album.title AS album_title
  FROM album
  JOIN artist ON artist.artist_id = album.artist_id
 WHERE artist.name = :artist_name
 ORDER BY 1, 2;
