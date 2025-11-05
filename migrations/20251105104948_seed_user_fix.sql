-- Add migration script here
UPDATE users
SET password_hash = '$argon2id$v=19$m=15000,t=2,p=1$PjOaO8A8sGZZvLjDw1aVQQ$lNHsqhqBrbv80GluDiGWdC7Fl9zSFFBy+8P/051z5a0'
WHERE user_id = 'ddf8994f-d522-4659-8d02-c1d479057be6';