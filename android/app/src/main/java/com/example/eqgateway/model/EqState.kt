package com.example.eqgateway.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject

/**
 * Kotlin representation of the EQ State v0.1 JSON payload
 * returned by the Rust FFI [processUserInput] function.
 */
@Serializable
data class EqState(
    val affect: AffectState = AffectState(),
    val cognitive: CognitiveState = CognitiveState(),
    val risk: RiskAssessment = RiskAssessment(),
    val context: ContextSummary = ContextSummary(),
    val session_id: String = "",
    val timestamp: String = "",
)

@Serializable
data class AffectState(
    val primary: String = "unknown",
    val secondary: List<String> = emptyList(),
    val valence: Double = 0.0,
    val arousal: Double = 0.0,
    val confidence: Double = 0.0,
)

@Serializable
data class CognitiveState(
    val intent: String = "unknown",
    val topics: List<String> = emptyList(),
    val urgency: Double = 0.0,
)

@Serializable
data class RiskAssessment(
    val level: String = "none",
    val sensitivity: List<String> = emptyList(),
    val requires_human: Boolean = false,
    val blocked: Boolean = false,
)

@Serializable
data class ContextSummary(
    val turn_count: Int = 0,
    val session_active_sec: Double = 0.0,
)

/**
 * Parses the raw JSON string from [processUserInput] into an [EqState].
 */
object EqStateParser {
    private val json = Json { ignoreUnknownKeys = true; coerceInputValues = true }

    fun parse(raw: String): EqState {
        return try {
            json.decodeFromString(raw)
        } catch (e: Exception) {
            // Fallback: try to extract what we can
            EqState(
                affect = AffectState(primary = "parse_error"),
                risk = RiskAssessment(level = "none"),
            )
        }
    }
}
