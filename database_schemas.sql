-- Key-value storage

CREATE TABLE IF NOT EXISTS "key_values" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"parent_id" INTEGER REFERENCES "key_values" ("id"),
	"key" TEXT NOT NULL,
	"value" TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS "index_key_values__ifnull_parent_id__key" ON "key_values" (IFNULL("parent_id", 0), "key");
CREATE INDEX IF NOT EXISTS "index_key_values__parent_id__key" ON "key_values" ("parent_id", "key");


-- Message queue storage

CREATE TABLE IF NOT EXISTS "message_paths" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"parent_id" INTEGER REFERENCES "message_paths",
	"name" TEXT NOT NULL,
);
CREATE UNIQUE INDEX IF NOT EXISTS "index_message_paths__ifnull_parent_id__name" on "message_paths" (IFNULL("parent_id", 0), "name");
CREATE INDEX IF NOT EXISTS "index_message_paths__parent_id__name" on "message_paths" ("parent_id", "name");

CREATE TABLE IF NOT EXISTS "messages" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"path_id" INTEGER NOT NULL REFERENCES "message_paths",
	"message_id" TEXT NOT NULL UNIQUE,
	"timestamp" TEXT NOT NULL,
	"message" TEXT NOT NULL,
);
CREATE INDEX IF NOT EXISTS "_index_messages__path_id" ON "messages" ("path_id");
CREATE INDEX IF NOT EXISTS "_index_messages__timestamp" ON "messages" ("timestamp");


-- Authentication storage

CREATE TABLE IF NOT EXISTS "users" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"username" TEXT NOT NULL UNIQUE,
	"password_hash" TEXT,
	"active" INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS "roles" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"name" TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS "user_roles" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"user_id" INTEGER NOT NULL REFERENCES "_users" ("id"),
	"role_id" INTEGER NOT NULL REFERENCES "_roles" ("id"),
	UNIQUE ("user_id", "role_id")
);

CREATE TABLE IF NOT EXISTS "sessions" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"token" TEXT NOT NULL UNIQUE,
	"user_id" INTEGER NOT NULL REFERENCES "_users" ("id"),
	"timestamp" TEXT NOT NULL,
);
