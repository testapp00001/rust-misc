-- ==========================================================================
-- Combined Atomic Purchase Script
-- ==========================================================================
-- This is THE master Lua script for the flash sale system.
-- It performs all four checks and all mutations atomically.
--
-- This file exists for reference. The actual Rust code embeds this script
-- as a const string literal (see p05_combined_purchase.rs).
-- ==========================================================================

-- KEYS:
--   KEYS[1] = stock key            (e.g. "stock:product:42")
--   KEYS[2] = claims set key       (e.g. "claims:product:42")
--   KEYS[3] = voucher counter key  (e.g. "vouchers:product:42")
--   KEYS[4] = idempotency key      (e.g. "idemp:req-abc-123")
--
-- ARGV:
--   ARGV[1] = account_id
--   ARGV[2] = max_vouchers (string-encoded integer)
--   ARGV[3] = request_id
--   ARGV[4] = ttl_seconds (string-encoded integer)
--
-- RETURN VALUE (Lua array):
--   {1, voucher_code}           -- Success
--   {2}                         -- SoldOut
--   {3}                         -- AlreadyClaimed
--   {4}                         -- VoucherLimitReached
--   {5, cached_result_string}   -- IdempotentReplay

-- --------------------------------------------------------------------------
-- Step 1: Idempotency check
-- --------------------------------------------------------------------------
-- If this exact request_id has been processed before, return the cached
-- result immediately. This handles network retries, client retries, and
-- message queue redelivery.
local existing = redis.call('GET', KEYS[4])
if existing then
    return {5, existing}  -- IdempotentReplay
end

-- --------------------------------------------------------------------------
-- Step 2: Account claim check
-- --------------------------------------------------------------------------
-- Each account may only purchase a given product once. SISMEMBER is O(1).
local already_claimed = redis.call('SISMEMBER', KEYS[2], ARGV[1])
if already_claimed == 1 then
    -- Store the result in the idempotency cache so the next call with the
    -- same request_id returns this directly (Step 1 catches it).
    redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'ALREADY_CLAIMED')
    return {3}  -- AlreadyClaimed
end

-- --------------------------------------------------------------------------
-- Step 3: Stock check and decrement
-- --------------------------------------------------------------------------
-- Read the current stock. If nil (key doesn't exist) or <= 0, sold out.
-- We decrement HERE, before checking the voucher limit. If the voucher
-- limit check fails we roll back the decrement (Step 4b).
local stock = redis.call('GET', KEYS[1])
if not stock or tonumber(stock) <= 0 then
    redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'SOLD_OUT')
    return {2}  -- SoldOut
end
redis.call('DECR', KEYS[1])

-- --------------------------------------------------------------------------
-- Step 4: Voucher limit check
-- --------------------------------------------------------------------------
-- Each product has a maximum number of discount vouchers. If the limit is
-- reached, we must ROLL BACK the stock decrement to keep the system
-- consistent.
local voucher_count = tonumber(redis.call('GET', KEYS[3]) or '0')
local max_vouchers = tonumber(ARGV[2])
if voucher_count >= max_vouchers then
    -- 4a: Roll back the stock decrement
    redis.call('INCR', KEYS[1])
    -- 4b: Store idempotency result
    redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'VOUCHER_LIMIT_REACHED')
    return {4}  -- VoucherLimitReached
end
redis.call('INCR', KEYS[3])

-- --------------------------------------------------------------------------
-- Step 5: Record the account claim
-- --------------------------------------------------------------------------
redis.call('SADD', KEYS[2], ARGV[1])

-- --------------------------------------------------------------------------
-- Step 6: Generate a voucher code
-- --------------------------------------------------------------------------
-- In production this would use a UUID or a dedicated sequence generator.
-- Here we use a simple deterministic format for easy testing.
local new_voucher_count = voucher_count + 1
local voucher_code = 'VCHR-' .. ARGV[1] .. '-' .. tostring(new_voucher_count)

-- --------------------------------------------------------------------------
-- Step 7: Store the idempotency result
-- --------------------------------------------------------------------------
redis.call('SETEX', KEYS[4], tonumber(ARGV[4]), 'SUCCESS:' .. voucher_code)

-- --------------------------------------------------------------------------
-- Return success
-- --------------------------------------------------------------------------
return {1, voucher_code}  -- Success
