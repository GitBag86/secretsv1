-- Remove session_hmac from app_settings on upgrade.
-- The HMAC is now stored only in memory (EncryptionManager.session_secret)
-- and computed on-the-fly during unlock/refresh. This avoids leaking the
-- session secret to disk.
DELETE FROM app_settings WHERE key = 'session_hmac';
