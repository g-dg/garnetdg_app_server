-- Key-value storage

CREATE TABLE IF NOT EXISTS "key_value_tree" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"parent_id" INTEGER REFERENCES "key_value_tree" ("id"),
	"key" TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS "index_key_value_tree__ifnull_parent_id__key" ON "key_value_tree" (IFNULL("parent_id", 0), "key");
CREATE INDEX IF NOT EXISTS "index_key_value_tree__parent_id__key" ON "key_value_tree" ("parent_id", "key");

CREATE TABLE IF NOT EXISTS "key_value" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"tree_node_id" INTEGER REFERENCES "key_value_tree" ("id"),
	"value" TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS "index_key_value__ifnull_tree_node_id" ON "key_value" (IFNULL("tree_node_id", 0));
CREATE INDEX IF NOT EXISTS "index_key_value__tree_node_id" ON "key_value" ("tree_node_id");


-- Message queue storage

CREATE TABLE IF NOT EXISTS "message_tree" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"parent_id" INTEGER REFERENCES "message_tree",
	"name" TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS "index_message_tree__ifnull_parent_id__name" on "message_tree" (IFNULL("parent_id", 0), "name");
CREATE INDEX IF NOT EXISTS "index_message_tree__parent_id__name" on "message_tree" ("parent_id", "name");

CREATE TABLE IF NOT EXISTS "messages" (
	"id" INTEGER PRIMARY KEY NOT NULL,
	"path_id" INTEGER NOT NULL REFERENCES "message_tree",
	"message_id" TEXT NOT NULL UNIQUE,
	"timestamp" TEXT NOT NULL,
	"message" TEXT NOT NULL
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
	"timestamp" TEXT NOT NULL
);
