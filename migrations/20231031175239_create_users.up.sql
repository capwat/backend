CREATE TABLE "users" (
    id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created_at timestamp NOT NULL DEFAULT(now() AT TIME ZONE 'utc'),
    name varchar(20) UNIQUE NOT NULL,
    email varchar(255) UNIQUE,
    display_name varchar(25),
    password_hash text NOT NULL,
    updated_at timestamp
);

    -- id            BigInt    @id @default(autoincrement())
    -- created_at    DateTime  @default(now()) @db.Timestamp()
    -- name          String    @db.VarChar(20)
    -- email         String?   @unique @db.VarChar(255)
    -- display_name  String?   @db.VarChar(25)
    -- password_hash String    @unique @db.Text
    -- updated_at    DateTime? @db.Timestamp()