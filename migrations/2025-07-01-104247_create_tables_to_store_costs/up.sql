-- Your SQL goes here
CREATE TABLE payments (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) NOT NULL,
  amount FLOAT NOT NULL,
  created_at DATE NOT NULL DEFAULT CURRENT_DATE
);

CREATE TABLE quotas (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users(id) NOT NULL,
  cost_id INTEGER REFERENCES costs(id) NOT NULL,
  percentage_quota FLOAT NOT NULL
);

CREATE TABLE payments_quotas (
  payment_id INTEGER REFERENCES payments(id) NOT NULL, 
  quota_id INTEGER REFERENCES quotas(id) NOT NULL, 
  PRIMARY KEY(payment_id, quota_id)
);

ALTER TABLE costs ADD COLUMN group_id INTEGER REFERENCES groups(id) NOT NULL DEFAULT 0;
ALTER TABLE costs ADD COLUMN user_id INTEGER REFERENCES users(id) NOT NULL DEFAULT 0;
ALTER TABLE costs ADD COLUMN amount FLOAT NOT NULL DEFAULT 0;