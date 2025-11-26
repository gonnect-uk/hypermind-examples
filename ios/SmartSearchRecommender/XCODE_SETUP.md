# Xcode Project Setup Guide

Complete instructions for setting up SmartSearchRecommender in Xcode.

## Method 1: Create New Project (Recommended)

### Step 1: Create Xcode Project

1. Open Xcode
2. **File → New → Project**
3. Choose template: **iOS → App**
4. Configure project:
   - **Product Name**: `SmartSearchRecommender`
   - **Team**: Your development team
   - **Organization Identifier**: `com.yourname` (or similar)
   - **Interface**: `SwiftUI`
   - **Language**: `Swift`
   - **Storage**: `None` (we'll use @Published properties)
   - **Include Tests**: ✅ Yes (recommended)
5. Save location: `/Users/gauravmalhotra/Documents/MyGithub/zenya-graphdb/rust-kgdb/ios/SmartSearchRecommender/`

### Step 2: Delete Default Files

Xcode creates these files - delete them:
- `ContentView.swift` (we have our own)
- `SmartSearchRecommenderApp.swift` (we'll replace it)

### Step 3: Add Project Files

Drag these directories from Finder into Xcode project navigator:

1. **SmartSearchRecommender/** (folder)
   - ✅ Check "Copy items if needed"
   - ✅ Check "Create groups"
   - ✅ Add to target: SmartSearchRecommender

This adds:
- `SmartSearchRecommenderApp.swift`
- `Models/` (3 files)
- `Services/` (2 files)
- `Views/` (5 files)

**Total: 11 Swift files**

### Step 4: Add Data File

1. Drag `../../test-data/movies_catalog.ttl` into project
2. ✅ Check "Copy items if needed"
3. ✅ Add to target: SmartSearchRecommender
4. Verify in **Build Phases → Copy Bundle Resources**

### Step 5: Configure Build Settings

#### General Tab
- **Minimum Deployments**: iOS 17.0
- **Device**: iPhone, iPad
- **Orientation**: Portrait (recommended)

#### Build Settings Tab
- **Swift Language Version**: Swift 5
- **Enable Bitcode**: No
- **Optimization Level** (Debug): Fast, Debug Friendly
- **Optimization Level** (Release): Optimize for Speed

#### Signing & Capabilities
- **Team**: Select your team
- **Signing Certificate**: Automatic
- **Bundle Identifier**: com.yourname.SmartSearchRecommender

### Step 6: Build & Run

1. Select scheme: **SmartSearchRecommender**
2. Select device: **iPhone 15 Pro** (Simulator)
3. Press **Cmd+R** to build and run

Expected result: App launches with movie discovery interface.

---

## Method 2: Use Workspace (Alternative)

If you already have an Xcode workspace:

### Step 1: Add to Workspace

1. Open existing workspace: `ios/GraphDBAdmin/GraphDBAdmin.xcworkspace`
2. Right-click workspace → **Add Files to "Workspace"**
3. Navigate to `ios/SmartSearchRecommender/`
4. Select folder, check "Create groups"

### Step 2: Set Dependencies

If using shared frameworks:
1. Select SmartSearchRecommender target
2. **General → Frameworks, Libraries, and Embedded Content**
3. Add: `RustKGDB.framework` (when available)

### Step 3: Configure Schemes

1. **Product → Scheme → Manage Schemes**
2. Add new scheme: "SmartSearchRecommender"
3. Set build targets

---

## File Structure in Xcode

After setup, your project navigator should look like:

```
SmartSearchRecommender
├── SmartSearchRecommender/
│   ├── SmartSearchRecommenderApp.swift
│   ├── Models/
│   │   ├── Movie.swift
│   │   ├── Person.swift
│   │   └── Genre.swift
│   ├── Services/
│   │   ├── MovieService.swift
│   │   └── RecommendationEngine.swift
│   └── Views/
│       ├── HomeView.swift
│       ├── MovieDetailView.swift
│       ├── SearchResultsView.swift
│       ├── PersonDetailView.swift
│       └── ExplainRecommendationView.swift
├── Resources/
│   └── movies_catalog.ttl
├── SmartSearchRecommenderTests/
│   └── SmartSearchRecommenderTests.swift
├── SmartSearchRecommenderUITests/
│   └── SmartSearchRecommenderUITests.swift
└── Products/
    └── SmartSearchRecommender.app
```

---

## Build Phases Configuration

Verify these build phases:

### 1. Compile Sources (11 files)
```
✅ SmartSearchRecommenderApp.swift
✅ Movie.swift
✅ Person.swift
✅ Genre.swift
✅ MovieService.swift
✅ RecommendationEngine.swift
✅ HomeView.swift
✅ MovieDetailView.swift
✅ SearchResultsView.swift
✅ PersonDetailView.swift
✅ ExplainRecommendationView.swift
```

### 2. Copy Bundle Resources (1 file)
```
✅ movies_catalog.ttl
```

### 3. Link Binary With Libraries
```
✅ SwiftUI.framework (automatic)
✅ Foundation.framework (automatic)
✅ Combine.framework (automatic)
```

---

## Info.plist Configuration

Add these keys if needed:

### Required
```xml
<key>UIApplicationSceneManifest</key>
<dict>
    <key>UIApplicationSupportsMultipleScenes</key>
    <false/>
</dict>
```

### Optional (for future features)
```xml
<!-- Photo Library (for sharing) -->
<key>NSPhotoLibraryUsageDescription</key>
<string>Save and share movie recommendations</string>

<!-- Camera (for AR features) -->
<key>NSCameraUsageDescription</key>
<string>Scan movie posters</string>
```

---

## Common Build Issues & Solutions

### Issue 1: "No such module 'SwiftUI'"
**Cause**: Wrong deployment target
**Solution**: Set minimum deployment to iOS 17.0

### Issue 2: "'async' can only be used in iOS 13.0 or newer"
**Cause**: Old deployment target
**Solution**: Target → General → Minimum Deployments → iOS 17.0

### Issue 3: "Cannot find 'MovieService' in scope"
**Cause**: File not added to target
**Solution**: Select file → File Inspector → Target Membership → ✅ SmartSearchRecommender

### Issue 4: "movies_catalog.ttl not found"
**Cause**: File not in bundle resources
**Solution**: Build Phases → Copy Bundle Resources → Add file

### Issue 5: "Ambiguous use of 'init'"
**Cause**: SwiftUI property wrapper issue
**Solution**: Clean build folder (Cmd+Shift+K) and rebuild

### Issue 6: "Command PhaseScriptExecution failed"
**Cause**: Rust framework build script
**Solution**: Remove build script if not using FFI yet

---

## Testing Setup

### Unit Tests

Create `SmartSearchRecommenderTests.swift`:

```swift
import XCTest
@testable import SmartSearchRecommender

final class MovieServiceTests: XCTestCase {
    var movieService: MovieService!

    override func setUp() {
        super.setUp()
        movieService = MovieService.shared
    }

    func testLoadMovies() async {
        await movieService.loadMoviesCatalog()
        XCTAssertGreaterThan(movieService.allMovies.count, 0)
    }

    func testSearchMovies() async {
        await movieService.loadMoviesCatalog()
        let results = await movieService.searchMovies(query: "Dark")
        XCTAssertTrue(results.contains { $0.title.contains("Dark") })
    }

    func testRecommendations() async {
        await movieService.loadMoviesCatalog()
        let movie = movieService.allMovies.first!
        let engine = RecommendationEngine(movieService: movieService)
        let recommendations = await engine.findSimilarMovies(to: movie)
        XCTAssertGreaterThan(recommendations.count, 0)
    }
}
```

Run: **Cmd+U**

### UI Tests

Create `SmartSearchRecommenderUITests.swift`:

```swift
import XCTest

final class SmartSearchRecommenderUITests: XCTestCase {
    var app: XCUIApplication!

    override func setUp() {
        super.setUp()
        continueAfterFailure = false
        app = XCUIApplication()
        app.launch()
    }

    func testHomeScreenAppears() {
        XCTAssertTrue(app.navigationBars["Discover"].exists)
    }

    func testSearchTab() {
        app.tabBars.buttons["Search"].tap()
        XCTAssertTrue(app.navigationBars["Search"].exists)
    }

    func testMovieDetail() {
        // Tap first movie
        let firstMovie = app.scrollViews.buttons.firstMatch
        firstMovie.tap()

        // Verify detail view
        XCTAssertTrue(app.staticTexts.element(matching: .any, identifier: "movieTitle").exists)
    }
}
```

Run: **Cmd+U** (with UI Tests scheme)

---

## Advanced Configuration

### 1. Add Rust KGDB Framework

When ready to integrate FFI:

1. Build Rust framework:
   ```bash
   cd ../..
   cargo build --release -p mobile-ffi
   ```

2. Copy to Xcode:
   ```bash
   cp target/universal/release/RustKGDB.framework ios/Frameworks/
   ```

3. Add to project:
   - Target → General → Frameworks, Libraries, and Embedded Content
   - Add → RustKGDB.framework
   - Embed & Sign

4. Uncomment FFI code in `MovieService.swift`

### 2. Enable Swift Concurrency Checks

Build Settings:
- **Swift Compiler - Custom Flags**
- **Other Swift Flags**: `-Xfrontend -warn-concurrency`

### 3. Enable Strict Checking

Build Settings:
- **Swift Compiler - Warnings**
- ✅ Treat Warnings as Errors (Release only)

### 4. Optimize for Size

For App Store submission:
- **Optimization Level** (Release): Optimize for Size
- **Strip Symbols**: Yes
- **Strip Unused Code**: Yes

---

## Xcode Shortcuts

### Essential
- **Cmd+R**: Run
- **Cmd+B**: Build
- **Cmd+U**: Test
- **Cmd+.**: Stop
- **Cmd+Shift+K**: Clean build folder

### Navigation
- **Cmd+1-9**: Show navigators
- **Cmd+0**: Show/hide navigator
- **Cmd+Option+0**: Show/hide inspector
- **Cmd+Shift+Y**: Show/hide debug area

### Editing
- **Cmd+/**: Comment/uncomment
- **Cmd+[**: Shift left
- **Cmd+]**: Shift right
- **Option+Cmd+[**: Move line up
- **Option+Cmd+]**: Move line down

### Preview
- **Option+Cmd+Return**: Show SwiftUI preview
- **Option+Cmd+P**: Resume preview

---

## Deployment Checklist

### Pre-Release
- [ ] Set version number (1.0.0)
- [ ] Set build number (1)
- [ ] Update bundle ID
- [ ] Configure signing
- [ ] Add app icon
- [ ] Add launch screen
- [ ] Test on device

### App Store
- [ ] Archive build
- [ ] Upload to App Store Connect
- [ ] Add screenshots
- [ ] Write description
- [ ] Set pricing
- [ ] Submit for review

---

## Next Steps

1. **Build the project** - Verify everything compiles
2. **Run on simulator** - Test basic navigation
3. **Test on device** - Verify performance
4. **Integrate FFI** - Connect to Rust KGDB
5. **Expand data** - Add more movies
6. **Submit to App Store** - Share with the world!

---

**Xcode Version**: 15.0+
**Swift Version**: 5.9+
**iOS Version**: 17.0+

**Ready to build!** 🚀
