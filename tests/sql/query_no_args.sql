-- name: get_top_artists?
-- Retrieves "top" artists, i.e. artists with 10 or more listed albums
SELECT artist.name AS artist_name, Count(*) AS num_albums
  FROM album
  JOIN artist ON artist.artist_id = album.artist_id
 GROUP BY artist.name
HAVING Count(*) > 10
 ORDER BY 2 DESC
