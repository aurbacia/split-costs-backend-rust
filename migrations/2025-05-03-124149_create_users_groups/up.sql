CREATE TABLE users_groups (
  group_id INTEGER REFERENCES groups(id),
  user_id INTEGER REFERENCES users(id),
  created_at DATE NOT NULL DEFAULT CURRENT_DATE,
  PRIMARY KEY(group_id, user_id)
)