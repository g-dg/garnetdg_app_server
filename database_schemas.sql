-- Datastore storage

CREATE TABLE IF NOT EXISTS "datastore_tree" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"parent_id" INTEGER REFERENCES "datastore_tree" ("id"),
	"key" TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS "index_datastore_tree__ifnull_parent_id__key" ON "datastore_tree" (IFNULL("parent_id", 0), "key");
CREATE INDEX IF NOT EXISTS "index_datastore_tree__parent_id__key" ON "datastore_tree" ("parent_id", "key");

CREATE TABLE IF NOT EXISTS "datastore_values" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"tree_node_id" INTEGER REFERENCES "datastore_tree",
	"change_id" TEXT NOT NULL UNIQUE,
	"timestamp" TEXT NOT NULL,
	"value" TEXT
);
CREATE INDEX IF NOT EXISTS "index_datastore_values__tree_node_id" ON "datastore_values" ("tree_node_id");
CREATE INDEX IF NOT EXISTS "index_datastore_values__timestamp" ON "datastore_values" ("timestamp");

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
	"timestamp" TEXT NOT NULL
);
