# npm Account Setup and Login Guide

## Step-by-Step: Creating npm Account

### Method 1: Via Website (Easiest)

1. **Go to npm website:**
   - Visit: https://www.npmjs.com/signup

2. **Fill in registration form:**
   - **Username:** Choose a unique username (lowercase, no spaces)
   - **Email:** Your email address
   - **Password:** Strong password (minimum 10 characters)

3. **Verify email:**
   - Check your email inbox
   - Click the verification link from npm
   - This is **required** before you can publish packages

4. **Complete setup:**
   - You now have an npm account!

### Method 2: Via Command Line

```bash
# Create account from terminal
npm adduser

# You'll be prompted for:
# Username: your-username
# Password: your-password
# Email: your@email.com
```

---

## Logging In to npm

### From Command Line

```bash
# Login to npm
npm login

# You'll be prompted for:
# Username: your-username
# Password: your-password
# Email: your@email.com (public)
# OTP (if 2FA enabled): 123456
```

### Verify You're Logged In

```bash
# Check current user
npm whoami

# Should display your username
# Example output: gauravmalhotra
```

---

## Setting Up Two-Factor Authentication (2FA) - HIGHLY RECOMMENDED

### Why Enable 2FA?
- **Required for publishing:** npm requires 2FA for package publishers
- **Security:** Protects your packages from hijacking
- **Industry standard:** All serious publishers use 2FA

### Enable 2FA

1. **Install authenticator app on your phone:**
   - Google Authenticator (iOS/Android)
   - Authy (iOS/Android)
   - 1Password (if you use it)

2. **Enable 2FA via website:**
   ```
   https://www.npmjs.com/settings/YOUR_USERNAME/tfa
   ```

3. **OR enable via command line:**
   ```bash
   npm profile enable-2fa auth-and-writes
   ```

4. **Scan QR code with your authenticator app**

5. **Save recovery codes** in a safe place!

### Using 2FA When Publishing

When you run `npm publish`, you'll be prompted:
```bash
npm notice Please enter OTP:
```

Enter the 6-digit code from your authenticator app.

---

## Publishing to Scoped Packages (@gonnect/rust-kgdb)

### Create npm Organization (Optional)

For `@gonnect/rust-kgdb`, you need either:
1. **npm Organization** named "zenya" (costs $7/month for private packages, free for public)
2. **Use your username** as scope: `@your-username/rust-kgdb`

#### Option A: Create Organization

1. Go to: https://www.npmjs.com/org/create
2. Organization name: `zenya`
3. Choose plan: **Free** (for unlimited public packages)
4. Complete setup

#### Option B: Use Personal Scope

Just publish as `@your-username/rust-kgdb` - no setup needed!

---

## Publishing Your First Package

### Pre-Publishing Checklist

```bash
# 1. Verify you're logged in
npm whoami

# 2. Check package name availability
npm search @gonnect/rust-kgdb
# OR
npm view @gonnect/rust-kgdb
# Should return 404 if available

# 3. Verify package.json
cat sdks/typescript/native/rust-kgdb-napi/package.json

# 4. Test package locally
cd sdks/typescript/native/rust-kgdb-napi
npm pack
# Creates: zenya-rust-kgdb-0.1.3.tgz

# 5. Dry run publish
npm publish --dry-run --access public
```

### Publish!

```bash
# For scoped package (@gonnect/rust-kgdb)
npm publish --access public

# For unscoped package (rust-kgdb)
npm publish
```

---

## Troubleshooting

### Error: "You must be logged in to publish packages"

**Solution:**
```bash
npm login
npm whoami  # Verify
```

### Error: "You do not have permission to publish"

**Causes:**
1. Package name already taken
2. Not logged in
3. Not a member of the organization

**Solutions:**
```bash
# Check if package exists
npm view @gonnect/rust-kgdb

# Check organization membership
npm org ls zenya

# Try different package name
# Change "name" in package.json
```

### Error: "This operation requires a one-time password"

**Solution:**
Enable 2FA and use OTP when publishing:
```bash
npm publish --otp=123456
```

### Error: "Package name too similar to existing package"

**Solution:**
Choose a more unique name. npm prevents typosquatting.

---

## Managing Access Tokens (Alternative to Password)

### Why Use Tokens?
- **CI/CD:** Automate publishing in GitHub Actions
- **Security:** Revocable without changing password
- **Granular:** Different tokens for different purposes

### Create Token

1. **Via website:**
   - Go to: https://www.npmjs.com/settings/YOUR_USERNAME/tokens
   - Click "Generate New Token"
   - Choose type: **Automation** (for CI/CD) or **Publish** (for manual)
   - Copy token (shown only once!)

2. **Via command line:**
   ```bash
   npm token create --read-only  # For downloading
   npm token create              # For publishing
   ```

### Use Token

```bash
# Set token in .npmrc (in your home directory)
echo "//registry.npmjs.org/:_authToken=YOUR_TOKEN_HERE" >> ~/.npmrc

# OR set as environment variable
export NPM_TOKEN=your-token-here
```

### For CI/CD (GitHub Actions)

```yaml
# .github/workflows/publish.yml
- name: Publish to npm
  run: npm publish --access public
  env:
    NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

---

## Quick Reference Commands

```bash
# Account Management
npm adduser                  # Create account
npm login                    # Login
npm logout                   # Logout
npm whoami                   # Check current user

# Package Management
npm publish                  # Publish package
npm unpublish pkg@version    # Remove version (within 72 hours)
npm deprecate pkg "message"  # Deprecate package

# Organization Management
npm org ls your-org          # List members
npm org set your-org user    # Add member

# Token Management
npm token list               # List tokens
npm token create             # Create token
npm token revoke <token>     # Revoke token

# Profile Management
npm profile get              # View profile
npm profile set key value    # Update profile
npm profile enable-2fa       # Enable 2FA
```

---

## Security Best Practices

1. ✅ **Enable 2FA** - Required for publishing
2. ✅ **Use strong password** - Minimum 10 characters
3. ✅ **Verify email** - Required before publishing
4. ✅ **Use tokens for CI/CD** - Not your password
5. ✅ **Review package before publishing** - Use `npm pack --dry-run`
6. ✅ **Sign packages** - Use `npm publish --provenance`
7. ✅ **Monitor downloads** - Watch for suspicious activity

---

## Publishing Checklist for @gonnect/rust-kgdb

- [ ] npm account created and verified
- [ ] Logged in: `npm whoami` works
- [ ] 2FA enabled (required for publishing)
- [ ] Organization "zenya" created (or using personal scope)
- [ ] Package name available: `npm view @gonnect/rust-kgdb` returns 404
- [ ] package.json has correct metadata
- [ ] Binary built: `cargo build --release -p rust-kgdb-napi`
- [ ] .npmignore excludes source code
- [ ] Dry run successful: `npm publish --dry-run --access public`
- [ ] Ready to publish: `npm publish --access public`

---

## Next Steps After Publishing

1. **Verify on npm:**
   - https://www.npmjs.com/package/@gonnect/rust-kgdb

2. **Test installation:**
   ```bash
   npm install @gonnect/rust-kgdb
   ```

3. **Monitor stats:**
   - https://npmcharts.com/@gonnect/rust-kgdb
   - https://npm-stat.com/charts.html?package=@gonnect/rust-kgdb

4. **Announce:**
   - Twitter/LinkedIn
   - GitHub Releases
   - Project website

---

## Resources

- **npm Documentation:** https://docs.npmjs.com/
- **Publishing Guide:** https://docs.npmjs.com/packages-and-modules/contributing-packages-to-the-registry
- **2FA Guide:** https://docs.npmjs.com/configuring-two-factor-authentication
- **npm Registry:** https://registry.npmjs.org/

---

**Last Updated:** November 30, 2025
**Version:** 0.1.3
