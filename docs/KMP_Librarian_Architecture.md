# KMP Librarian Architecture — The Bridge

**Status:** Design Draft  
**Version:** 0.1  
**Date:** June 08, 2026  
**Layer:** Logic Layer (The Bridge — Kotlin Multiplatform)

---

## 1. Purpose

The Librarian is the **on-device memory and context manager**. It lives in the KMP shared logic layer and is the only component that maintains state between user sessions. Its core responsibilities:

1. **Store user preferences** locally (tone preferences, privacy defaults, feature toggles).
2. **Manage conversation history** as rolling EQ State summaries.
3. **Provide context** to the Rust Engine for anonymized summary generation.
4. **Track longitudinal patterns** (e.g., "user has been frustrated for 3 days").
5. **Never sync anything to the cloud** — all data stays in an encrypted local store.

---

## 2. Data Principles

| Principle | Rule |
|:---|:---|
| **Local First** | All Librarian data is stored on-device. No cloud backup. No sync. |
| **Encrypted at Rest** | SQLite database is encrypted via SQLCipher (or Android EncryptedSharedPreferences). |
| **Ephemeral by Default** | Conversation history expires after 30 days (configurable per user). |
| **User Controlled** | User can view, export, or delete all Librarian data from the Privacy Dashboard. |
| **Minimum Viable** | Only store what's needed for the EQ State to function. No clickstream, no analytics. |

---

## 3. Module Structure

```
shared/
├── src/
│   ├── commonMain/
│   │   └── kotlin/com/eqgateway/librarian/
│   │       ├── Librarian.kt              # Public API facade
│   │       ├── models/
│   │       │   ├── EQStateSummary.kt     # Lightweight EQ State for history
│   │       │   ├── UserPreferences.kt    # User settings and defaults
│   │       │   ├── ConversationTurn.kt   # A single user↔AI interaction record
│   │       │   └── PrivacyDashboard.kt   # View models for privacy UI
│   │       ├── store/
│   │       │   ├── DatabaseDriverFactory.kt  # Platform-specific DB driver
│   │       │   ├── PreferenceStore.kt        # Key-value preference persistence
│   │       │   ├── ConversationStore.kt      # CRUD for conversation history
│   │       │   ├── EQStateStore.kt           # Time-series EQ State storage
│   │       │   └── MigrationManager.kt       # Schema migration for local DB
│   │       ├── privacy/
│   │       │   ├── RetentionPolicy.kt        # TTL and auto-purge logic
│   │       │   ├── ExportEngine.kt           # User data export (JSON/CSV)
│   │       │   └── AuditLogger.kt            # Local-only audit trail
│   │       └── util/
│   │           ├── SessionIdGenerator.kt     # UUID v4 for ephemeral sessions
│   │           └── TimeProvider.kt           # Testable time abstraction
│   │
│   ├── androidMain/
│   │   └── kotlin/com/eqgateway/librarian/
│   │       └── store/
│   │           └── DatabaseDriverFactory.android.kt  # Android SQLCipher driver
│   │
│   └── iosMain/
│       └── kotlin/com/eqgateway/librarian/
│           └── store/
│               └── DatabaseDriverFactory.ios.kt      # iOS SQLCipher via Native
```

---

## 4. Core Models

### 4.1 EQStateSummary (Lightweight History Record)

```kotlin
// models/EQStateSummary.kt

/**
 * A lightweight, storable version of the EQ State.
 * Designed for local persistence — not every field from the
 * full EQ State is needed for historical pattern analysis.
 *
 * Stored in the local SQLite database.
 */
data class EQStateSummary(
    /** Ephemeral session ID this state belongs to */
    val sessionId: String,

    /** ISO-8601 timestamp of when this state was captured */
    val capturedAt: String,

    /** Primary affect label (for time-series trending) */
    val affectPrimary: String,

    /** Valence score for mood trending */
    val valence: Double,

    /** Arousal score for energy trending */
    val arousal: Double,

    /** Intent category for usage analysis */
    val intentCategory: String,

    /** Risk level for escalation tracking */
    val riskLevel: String,

    /** Privacy sensitivity level */
    val privacySensitivity: String,

    /** Length of the anonymized summary in tokens */
    val summaryTokenCount: Int,

    /** Schema version for forward compatibility */
    val schemaVersion: String
)
```

### 4.2 ConversationTurn

```kotlin
// models/ConversationTurn.kt

/**
 * A single user↔AI interaction, stored locally for context retrieval.
 *
 * The RAW user text is NEVER stored here. Only the anonymized summary
 * and the EQ State metadata are persisted.
 *
 * Retention: Auto-purged after 30 days (configurable).
 */
data class ConversationTurn(
    /** Unique ID for this turn */
    val turnId: String,

    /** Ephemeral session ID */
    val sessionId: String,

    /** When this turn occurred */
    val timestamp: String,

    /** Whether this was a user message or AI response */
    val role: String,  // "user" | "assistant"

    /**
     * The anonymized, PII-free summary of the user's input.
     * This is what gets provided to the Large AI for context.
     * RAW TEXT IS NEVER STORED HERE.
     */
    val anonymizedContent: String,

    /** The EQ State metadata at the time of this turn */
    val eqState: EQStateSummary,

    /** The AI's response (stored for context window management) */
    val aiResponse: String?,

    /** Whether the user provided explicit feedback on the AI response */
    val userFeedback: String?  // null | "positive" | "neutral" | "negative"
)
```

### 4.3 UserPreferences

```kotlin
// models/UserPreferences.kt

/**
 * User-configurable preferences for the EQ Gateway.
 * All values are stored locally and NEVER transmitted to the cloud.
 *
 * Defaults are set for maximum privacy protection.
 */
data class UserPreferences(
    // -- Privacy Settings --
    /** Default privacy sensitivity tier (0-3) */
    val defaultPrivacyTier: Int = 1,  // "Metadata Only"

    /** Whether to allow the "request_raw_excerpt" prompt */
    val allowRawExcerptPrompts: Boolean = true,

    /** Auto-purge conversation history after N days (0 = never) */
    val retentionDays: Int = 30,

    // -- Response Preferences --
    /** Preferred communication tone */
    val preferredTone: String = "calm_direct",

    /** Preferred response length */
    val preferredLength: String = "medium",

    // -- Feature Toggles --
    /** Enable mood tracking over time */
    val enableLongitudinalTracking: Boolean = true,

    /** Show the privacy dashboard in the app menu */
    val showPrivacyDashboard: Boolean = true,

    // -- System --
    /** Last schema version migrated to */
    val schemaVersion: String = "0.2"
)
```

---

## 5. Storage Layer

### 5.1 Database Schema (SQLite via SQLDelight)

```sql
-- store/librarian.sq

-- Stores user preferences as a single-row key-value table
CREATE TABLE UserPreferences (
    id INTEGER PRIMARY KEY DEFAULT 1,
    default_privacy_tier INTEGER NOT NULL DEFAULT 1,
    allow_raw_excerpt_prompts INTEGER NOT NULL DEFAULT 1,
    retention_days INTEGER NOT NULL DEFAULT 30,
    preferred_tone TEXT NOT NULL DEFAULT 'calm_direct',
    preferred_length TEXT NOT NULL DEFAULT 'medium',
    enable_longitudinal_tracking INTEGER NOT NULL DEFAULT 1,
    show_privacy_dashboard INTEGER NOT NULL DEFAULT 1,
    schema_version TEXT NOT NULL DEFAULT '0.2',
    updated_at TEXT NOT NULL
);

-- Lightweight EQ State history (for mood trending)
CREATE TABLE EQStateHistory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    affect_primary TEXT NOT NULL,
    valence REAL NOT NULL,
    arousal REAL NOT NULL,
    intent_category TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    privacy_sensitivity TEXT NOT NULL,
    summary_token_count INTEGER NOT NULL,
    schema_version TEXT NOT NULL
);

CREATE INDEX idx_eq_history_captured_at ON EQStateHistory(captured_at);
CREATE INDEX idx_eq_history_session ON EQStateHistory(session_id);

-- Conversation turns (ring buffer, auto-purged by retention policy)
CREATE TABLE ConversationHistory (
    turn_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    anonymized_content TEXT NOT NULL,
    eq_state_id INTEGER REFERENCES EQStateHistory(id),
    ai_response TEXT,
    user_feedback TEXT CHECK(user_feedback IS NULL OR user_feedback IN ('positive', 'neutral', 'negative'))
);

CREATE INDEX idx_conversation_timestamp ON ConversationHistory(timestamp);
CREATE INDEX idx_conversation_session ON ConversationHistory(session_id);
```

### 5.2 Encryption

| Platform | Mechanism | Implementation |
|:---|:---|:---|
| **Android** | SQLCipher via `android.database.sqlite` | Database file encrypted with AES-256; key derived from Android Keystore |
| **iOS** | SQLCipher via Community Edition | Database file encrypted with AES-256; key stored in iOS Keychain |
| **Key management** | Biometric + device lock | Database unlocks only while the app is in the foreground and the device is unlocked |

---

## 6. Public API

```kotlin
// Librarian.kt

/**
 * The Librarian is the single entry point for all local data operations.
 *
 * All methods are suspending (async) — the underlying SQLite database
 * runs on a dedicated dispatcher to avoid blocking the UI thread.
 *
 * RAW USER TEXT IS NEVER ACCEPTED OR STORED BY THE LIBRARIAN.
 * Only EQ State summaries and anonymized content cross this boundary.
 */
class Librarian(
    private val dbDriver: SqlDriver,
    private val timeProvider: TimeProvider = DefaultTimeProvider()
) {
    // ------------------------------------------------------------------
    // Preferences
    // ------------------------------------------------------------------

    /** Load the user's saved preferences (or defaults if first launch). */
    suspend fun loadPreferences(): UserPreferences

    /** Persist updated user preferences atomically. */
    suspend fun savePreferences(prefs: UserPreferences)

    // ------------------------------------------------------------------
    // EQ State History
    // ------------------------------------------------------------------

    /** Record a new EQ State summary into the local time-series store. */
    suspend fun recordEQState(summary: EQStateSummary)

    /** Retrieve EQ State history for a date range (for mood trending UI). */
    suspend fun getEQStateHistory(
        fromDate: String,
        toDate: String
    ): List<EQStateSummary>

    /** Get aggregate mood data for the privacy dashboard. */
    suspend fun getMoodTrend(): MoodTrendData

    // ------------------------------------------------------------------
    // Conversation History
    // ------------------------------------------------------------------

    /** Store a conversation turn (user message or AI response). */
    suspend fun recordTurn(turn: ConversationTurn)

    /** Retrieve context window for the Large AI (most recent N turns). */
    suspend fun getContextWindow(
        sessionId: String,
        maxTurns: Int = 10
    ): List<ConversationTurn>

    /** Get all session IDs that have data (for session picker UI). */
    suspend fun getSessionList(): List<SessionInfo>

    // ------------------------------------------------------------------
    // Privacy & Retention
    // ------------------------------------------------------------------

    /** Purge records older than the retention policy. Called on app start. */
    suspend fun applyRetentionPolicy()

    /** Delete all data for a specific session. */
    suspend fun deleteSession(sessionId: String)

    /** Delete ALL local data (factory reset). */
    suspend fun wipeAllData()

    /** Export user data as JSON (for the Privacy Dashboard "Download My Data" feature). */
    suspend fun exportUserData(): String
}
```

---

## 7. Context Window Assembly

This is the critical path for the MCP `get_anonymized_context` tool.

```kotlin
// Librarian.kt (context assembly)

/**
 * Assembles a context window for the Large AI.
 *
 * Strategy:
 * 1. Load the most recent N turns from the current session.
 * 2. Load the EQ State summaries for those turns.
 * 3. If the session context exceeds the token budget, use
 *    a rolling summary technique:
 *    - Oldest turns → compressed to a single summary line
 *    - Recent turns → kept at full anonymized detail
 * 4. Return the assembled context WITH the EQ State timeline.
 */
suspend fun assembleContext(
    sessionId: String,
    maxTokens: Int = 1000,
    recentTurnCount: Int = 5
): AssembledContext {

    val recentTurns = getRecentTurns(sessionId, recentTurnCount)
    val olderTurns = getOlderTurns(sessionId, recentTurnCount)

    val summary = if (olderTurns.isNotEmpty()) {
        compressTurns(olderTurns, maxTokens / 2)
    } else null

    return AssembledContext(
        recentTurns = recentTurns,
        priorSummary = summary,
        eqStateTimeline = recentTurns.map { it.eqState },
        estimatedTokens = estimateTokens(recentTurns, summary)
    )
}

/**
 * Compression algorithm for old turns.
 * Concatenates anonymized summaries into a single paragraph,
 * then prepends an affect trend line.
 *
 * Example output:
 * "Prior context (3 turns): User was frustrated → anxious → calm.
 *  Topics discussed: workplace conflict, scheduling, coping strategies."
 */
private fun compressTurns(
    turns: List<ConversationTurn>,
    budget: Int
): String { ... }
```

---

## 8. Longitudinal Tracking

The Librarian can detect mood patterns over time without sending data to the cloud:

```kotlin
// privacy/MoodTrendEngine.kt

/**
 * Local-only mood pattern detection.
 * All computation happens on-device. No data leaves the Librarian.
 *
 * This enables the app UI to show things like:
 * "You've seemed frustrated for the last 3 days. Would you like to
 *  adjust your response tone to be more gentle?"
 */
class MoodTrendEngine(private val store: EQStateStore) {

    /**
     * Calculate the moving average of valence over the last N days.
     * Returns a list of (date, avgValence) pairs for charting.
     */
    suspend fun getValenceTrend(days: Int = 7): List<Pair<String, Double>>

    /**
     * Detect if the user's affect has been consistently negative
     * over the threshold period.
     */
    suspend fun detectPersistentDistress(
        thresholdDays: Int = 3,
        valenceBelow: Double = -0.3
    ): Boolean

    /**
     * Get the most frequently detected intent categories
     * for the current period. Used to personalize the
     * response policy defaults.
     */
    suspend fun getDominantIntentCategories(
        sinceDays: Int = 7
    ): Map<String, Int>
}
```

---

## 9. Compliance & Privacy Dashboard Integration

The Librarian powers the app's Privacy Dashboard — the "What happened to my message" view promised in the design doc.

```kotlin
// models/PrivacyDashboard.kt

/**
 * View model for the Privacy Dashboard UI.
 * The user can see every EQ State that was generated,
 * what was sent to the cloud, and what was kept local.
 *
 * This is the core transparency feature for PIPEDA/GDPR compliance.
 */
data class PrivacyDashboardData(
    /** Total number of EQ States generated since install */
    val totalStatesGenerated: Int,

    /** How many times raw text was shared (Tier 3 events) */
    val rawTextSharedCount: Int,

    /** List of Tier 3 events with user approval status */
    val rawTextSharedEvents: List<RawTextShareEvent>,

    /** Current privacy tier setting */
    val currentTier: Int,

    /** Storage used by the Librarian database */
    val storageUsedKB: Long,

    /** Number of conversation turns stored */
    val storedTurnCount: Int,

    /** Mood trend summary for the last 7 days */
    val moodTrend: MoodTrendData,

    /** Retention policy status */
    val retentionDaysRemaining: Int,

    /** When the last data export was performed */
    val lastExportTimestamp: String?
)

data class RawTextShareEvent(
    val timestamp: String,
    val reason: String,
    val excerptPreview: String,
    val userApproved: Boolean,
    val aiResponsePreview: String
)
```

---

## 10. Testing Strategy

### 10.1 Unit Tests

| Test | Focus |
|:---|:---|
| `LibrarianPreferencesTest` | Save/load/overwrite preferences; verify defaults on first launch |
| `LibrarianConversationTest` | CRUD for conversation turns; verify retention purge |
| `LibrarianEQStateTest` | Time-series storage; date range queries; aggregation |
| `MoodTrendEngineTest` | Moving average calculation; persistent distress detection |
| `ContextAssemblyTest` | Token budget enforcement; rolling summary generation |
| `RetentionPolicyTest` | Verify auto-purge deletes records older than TTL |
| `EncryptionTest` | Verify database is unreadable without the key (integration) |

### 10.2 Platform-Specific Tests

| Test | Focus |
|:---|:---|
| `AndroidEncryptionTest` | Verify SQLCipher + Android Keystore integration |
| `iOSEncryptionTest` | Verify SQLCipher + iOS Keychain integration |
| `BackgroundWipeTest` | Verify data wipe completes even if app is backgrounded |

---

## 11. Dependencies

```toml
# build.gradle.kts (shared module)

kotlin {
    sourceSets {
        commonMain {
            dependencies {
                // Database
                implementation("app.cash.sqldelight:runtime:2.0.1")
                implementation("app.cash.sqldelight:coroutines-extensions:2.0.1")

                // Serialization
                implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.2")

                // Date/time
                implementation("org.jetbrains.kotlinx:kotlinx-datetime:0.5.0")

                // Coroutines
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
            }
        }

        androidMain {
            dependencies {
                // SQLCipher for Android
                implementation("app.cash.sqldelight:android-driver:2.0.1")
                implementation("net.zetetic:android-database-sqlcipher:4.5.6")
            }
        }

        iosMain {
            dependencies {
                // SQLCipher for iOS
                implementation("app.cash.sqldelight:native-driver:2.0.1")
            }
        }
    }
}
```

---

*Confidential — EQ Gateway Project*
