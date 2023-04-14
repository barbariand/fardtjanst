BEGIN;
PRAGMA writable_schema = on;
PRAGMA encoding = 'UTF-8';
PRAGMA page_size = '4096';
PRAGMA auto_vacuum = '0';
PRAGMA user_version = '0';
PRAGMA application_id = '0';
CREATE TABLE sqlite_sequence(name,seq);
CREATE TABLE "tempsessions" (
	"id"	INTEGER,
	"user_id"	INTEGER,
	FOREIGN KEY("user_id") REFERENCES "users"("id"),
	PRIMARY KEY("id")
);
CREATE TABLE "resor" (
	"from_addres"	TEXT,
	"to_addres"	TEXT,
	"by_addres"	TEXT,
	"id"	INTEGER,
	"user_id"	INTEGER,
	"time"	INTEGER,
	"passagers"	INTEGER,
	"child_passagers"	INTEGER,
	"is_shared"	INTEGER,
	"can_be_new_trip_template"	INTEGER,
	"transport"	INTEGER,
	"from_id"	INTEGER,
	"by_id"	INTEGER,
	"to_id"	INTEGER,
	"cancelleable"	INTEGER,
	"company_name"	TEXT,
	"status"	TEXT,
	PRIMARY KEY("id" AUTOINCREMENT),
	FOREIGN KEY("user_id") REFERENCES "users"("id")
);
CREATE TABLE "users" (
	"id"	INTEGER,
	"name"	TEXT,
	"password"	TEXT,
	"card_nummer"	INTEGER,
	"phone_number"	TEXT,
	PRIMARY KEY("id" AUTOINCREMENT)
);
INSERT OR IGNORE INTO 'resor'('from_addres', 'to_addres', 'by_addres', 'id', 'user_id', 'time', 'passagers', 'child_passagers', 'is_shared', 'can_be_new_trip_template', 'transport', 'from_id', 'by_id', 'to_id', 'cancelleable', 'company_name', 'status') VALUES ('Nacka Gymnasium', 'Backamovägen 7', NULL, 1, 1, 1679921400000, 1, 0, 0, 0, 'Taxi', 21000000, NULL, 21000000, 0, NULL, 'Bye Bye');
INSERT OR IGNORE INTO 'users'('id', 'name', 'password', 'card_nummer', 'phone_number') VALUES (1, 'Test', '$2b$12$rWIqiE3Bp1Kex.KWdCeanOiqPAyur5w4UaN7xvksp00/q4GBvmON.', 14142855, '0702239460');
DELETE FROM sqlite_sequence;
INSERT OR IGNORE INTO 'sqlite_sequence'(_rowid_, 'name', 'seq') VALUES (5, 'resor', 1);
INSERT OR IGNORE INTO 'sqlite_sequence'(_rowid_, 'name', 'seq') VALUES (6, 'users', 1);
PRAGMA writable_schema = off;
COMMIT;
