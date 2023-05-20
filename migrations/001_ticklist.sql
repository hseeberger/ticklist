CREATE TABLE
  IF NOT EXISTS crag (
    id uuid PRIMARY KEY,
    name VARCHAR(256) NOT NULL,
    location VARCHAR(256) NOT NULL
  );

CREATE TABLE
  IF NOT EXISTS route (
    id uuid PRIMARY KEY,
    crag_id uuid REFERENCES crag ON DELETE RESTRICT,
    name VARCHAR(256) NOT NULL
  );

CREATE TABLE
  IF NOT EXISTS ascent (
    id uuid PRIMARY KEY,
    route_id uuid REFERENCES route ON DELETE RESTRICT,
    date date NOT NULL
  );