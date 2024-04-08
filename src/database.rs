// let pool = sqlx::postgres::PgPoolOptions::new()
//   .max_connections(1)
//   .connect("postgres://avnadmin:AVNS_vcH6CYuY4vN7Ayg8DoB@pg-1c46544a-tymonek12345-153d.a.aivencloud.com:25654/defaultdb?sslmode=require").await.unwrap();
//
// let row: (i64, ) = sqlx::query_as("SELECT $1")
//   .bind(150_i64)
//   .fetch_one(&pool).await.unwrap();
//
// println!("{:?}", row);