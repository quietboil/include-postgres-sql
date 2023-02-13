-- name: get_top_sales%
-- Retrieves artists with the most sales in a given city
-- # Parameters
-- param: city: &str - Customer's city
-- param: min_sold: i64 - mininum number of track sales
SELECT artist.name AS artist_name, track.name AS track_name, Sum(invoice_line.quantity) AS num_sold
  FROM invoice_line
  JOIN invoice  ON invoice.invoice_id = invoice_line.invoice_id
  JOIN customer ON customer.customer_id = invoice.customer_id
  JOIN track    ON track.track_id = invoice_line.track_id
  JOIN album    ON album.album_id = track.album_id
  JOIN artist   ON artist.artist_id = album.artist_id  
 WHERE customer.city = :city
 GROUP BY artist.name, track.name
HAVING Sum(invoice_line.quantity) >= :min_sold
 ORDER BY 1, 3
