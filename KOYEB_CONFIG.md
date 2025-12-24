# Koyeb Dockerfile Configuration

## Recommended Settings for API Deployment

Based on the Koyeb configuration interface, here are the settings you should use:

### Option 1: Use Dockerfile.api (Recommended)

**Dockerfile location:**
- Value: `Dockerfile.api`
- Override: **ON** (enable it)

**Entrypoint:**
- Value: (leave empty - already set in Dockerfile.api)
- Override: **OFF**

**Command:**
- Value: (leave empty - already set in Dockerfile.api)
- Override: **OFF** (turn it off)

**Target:**
- Value: (leave empty)
- Override: **OFF**

**Work directory:**
- Value: (leave empty)
- Override: **OFF**

### Option 2: Use Regular Dockerfile with Override

If you want to use the regular `Dockerfile` but run the API:

**Dockerfile location:**
- Value: `Dockerfile`
- Override: **OFF**

**Entrypoint:**
- Value: `matmul-api`
- Override: **ON** (enable it)

**Command:**
- Value: (leave empty)
- Override: **OFF** (turn it off)

**Target:**
- Value: (leave empty)
- Override: **OFF**

**Work directory:**
- Value: (leave empty)
- Override: **OFF**

## Step-by-Step Configuration

1. **Dockerfile location:**
   - Click the "Override" toggle to **ON**
   - Enter: `Dockerfile.api`
   - This tells Koyeb to use the API-specific Dockerfile

2. **Entrypoint:**
   - Leave override **OFF**
   - The Dockerfile.api already sets the entrypoint to `matmul-api`

3. **Command:**
   - Make sure override is **OFF** (turn it off if it's on)
   - The API server doesn't need additional command arguments

4. **Target:**
   - Leave override **OFF**
   - Not needed for this deployment

5. **Work directory:**
   - Leave override **OFF**
   - Not needed for this deployment

## Environment Variables

Also make sure to set these environment variables in Koyeb (if not already set):

- `PORT` - Koyeb usually sets this automatically, but you can set it to `8080` if needed
- `RUST_LOG` - Optional, set to `info` for logging

## Verification

After deployment, test your API:

```bash
# Health check
curl https://your-app.koyeb.app/health

# Test solve endpoint
curl -X POST https://your-app.koyeb.app/solve \
  -H "Content-Type: application/json" \
  -d '{
    "matrix_a": [[1.0, 2.0], [3.0, 4.0]],
    "matrix_b": [[5.0, 6.0], [7.0, 8.0]],
    "precision": "fp32"
  }'
```

## Quick Reference

**Simplest Configuration:**
- ✅ Dockerfile location: `Dockerfile.api` (Override: ON)
- ❌ Everything else: Override OFF, leave empty

This is the easiest setup since `Dockerfile.api` already has everything configured!

