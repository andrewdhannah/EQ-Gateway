/**
 * @file eq-state.schema.ts
 * @description The definitive TypeScript/Zod schema for the EQ Gateway "EQ State" payload.
 * This file serves as the "Source of Truth" for all components (Mobile, Rust Engine, Cloud AI).
 * 
 * VERSION: 0.1
 * STATUS: Finalized / Strict Implementation
 * 
 * ============================================================================
 * COMPREHENSIVE VARIABLE & TYPE SUMMARY
 * ============================================================================
 * 
 * --- ENUMS (Controlled Vocabularies) ---
 * [AffectPrimary]      : Valid primary emotional descriptors for the SLM.
 * [IntentCategory]     : High-level classification of user objectives.
 * [RiskLevel]          : Safety and urgency classifications.
 * [PrivacySensitivity] : Thresholds for data exposure and redaction.
 * [ResponseTone]       : Stylistic instructions for the Large AI persona.
 * 
 * --- SCHEMAS (Zod Validation Objects) ---
 * [AffectStateSchema]  : Metrics for valence, arousal, and primary/secondary emotion.
 * [IntentStateSchema]  : Details on intent category, subtype, and confidence.
 * [RiskStateSchema]    : Indicators for safety escalation and risk levels.
 * [PrivacyStateSchema] : Metadata on PII redaction and sensitivity tiers.
 * [ResponsePolicySchema]: Constraints on tone, warmth, directness, and length.
 * [ContextStateSchema] : The sanitized, anonymized summary of the dialogue.
 * [EQStateSchema]      : The root master schema combining all sub-states.
 * 
 * --- EXPORTED TYPES ---
 * [EQState]            : The TypeScript type inferred from EQStateSchema.
 * 
 * ============================================================================
 */

import { z } from 'zod';

// ----------------------------------------------------------------------------
// 1. CONTROLLED VOCABULARIES (Enums)
// ----------------------------------------------------------------------------

/**
 * @constant AffectPrimary
 * @description Valid primary emotional descriptors used by the SLM to classify state.
 */
export const AffectPrimary = z.enum([
  'neutral', 'calm', 'curious', 'pleased', 'hopeful', 'confused', 
  'uncertain', 'frustrated', 'angry', 'sad', 'anxious', 'overwhelmed', 
  'fatigued', 'embarrassed', 'lonely', 'excited', 'urgent', 'unknown'
]);

/**
 * @constant IntentCategory
 * @description High-level classification of user intent detected by the EQ Engine.
 */
export const IntentCategory = z.enum([
  'practical_guidance', 'emotional_support', 'decision_support', 'venting', 
  'planning', 'clarification', 'conflict_navigation', 'reflection', 
  'task_execution', 'creative_help', 'technical_help', 'safety_related', 'unknown'
]);

/**
 * @constant RiskLevel
 * @description Safety urgency levels determined by local risk detection algorithms.
 */
export const RiskLevel = z.enum([
  'none', 'low', 'medium', 'high', 'crisis', 'unknown'
]);

/**
 * @constant PrivacySensitivity
 * @description Threshold for how much data is permitted to leave the device.
 */
export const PrivacySensitivity = z.enum([
  'public', 'low', 'medium', 'high', 'restricted', 'unknown'
]);

/**
 * @constant ResponseTone
 * @description Stylistic instructions passed to the Large AI to shape its persona.
 */
export const ResponseTone = z.enum([
  'calm_direct', 'gentle_direct', 'warm_brief', 'neutral_professional', 
  'highly_practical', 'reflective', 'encouraging', 'minimal', 'unknown'
]);

// ----------------------------------------------------------------------------
// 2. SUB-SCHEMA DEFINITIONS (Nested Objects)
// ----------------------------------------------------------------------------

/**
 * @schema AffectStateSchema
 * @description Quantitative and qualitative measures of current emotion.
 */
const AffectStateSchema = z.object({
  /** The dominant emotion label */
  primary: AffectPrimary, 
  /** Nuanced secondary emotions detected in the text */
  secondary: z.array(AffectPrimary), 
  /** Valence score: -1.0 (extreme negative) to 1.0 (extreme positive) */
  valence: z.number().min(-1).max(1), 
  /** Arousal score: 0.0 (low energy/calm) to 1.0 (high energy/excitement) */
  arousal: z.number().min(0).max(1), 
  /** Model's statistical confidence in the affect classification (0 to 1) */
  confidence: z.number().min(0).max(1), 
  /** The method used to derive the signal (e.g., 'semantic_inference') */
  evidence_type: z.string() 
});

/**
 * @schema IntentStateSchema
 * @description Classification of what the user is attempting to achieve.
 */
const IntentStateSchema = z.object({
  /** The broad category of intent */
  category: IntentCategory, 
  /** A more granular description of the user's specific goal */
  subtype: z.string(), 
  /** Model's statistical confidence in intent classification (0 to 1) */
  confidence: z.number().min(0).max(1) 
});

/**
 * @schema RiskStateSchema
 * @description Safety assessment to trigger local escalation or cloud safety protocols.
 */
const RiskStateSchema = z.object({
  /** Current urgency/safety level */
  level: RiskLevel, 
  /** Specific linguistic or semantic indicators triggering this level */
  signals: z.array(z.string()), 
  /** Model's statistical confidence in risk assessment (0 to 1) */
  confidence: z.number().min(0).max(1), 
  /** Flag indicating whether the system must bypass cloud calls for safety */
  requires_local_escalation: z.boolean() 
});

/**
 * @schema PrivacyStateSchema
 * @description Metadata regarding data sanitization and disclosure limits.
 */
const PrivacyStateSchema = z.object({
  /** Determined sensitivity level of current conversation context */
  sensitivity_level: PrivacySensitivity, 
  /** Boolean: True if raw, unredacted text is being sent to the cloud */
  raw_text_shared: z.boolean(), 
  /** Boolean: Confirmation that PII has been successfully removed/masked */
  pii_removed: z.boolean(), 
  /** List of domains identified as sensitive (e.g., ['workplace']) */
  sensitive_domains_detected: z.array(z.string()), 
  /** Model's confidence in the redaction accuracy (0 to 1) */
  redaction_confidence: z.number().min(0).max(1) 
});

/**
 * @schema ResponsePolicySchema
 * @description Behavioral constraints passed to the Large AI via MCP tools.
 */
const ResponsePolicySchema = z.object({
  /** Desired tone of the AI's response */
  tone: ResponseTone, 
  /** Degree of warmth/empathy (0.0 to 1.0) */
  warmth: z.number().min(0).max(1), 
  /** Degree of directness/brevity (0.0 to 1.0) */
  directness: z.number().min(0).max(1), 
  /** Target response length: 'short', 'medium', or 'long' */
  length: z.enum(['short', 'medium', 'long']), 
  /** Expected conversational pace (e.g., 'steady', 'measured') */
  pace: z.string(), 
  /** Hard limit on the number of follow-up questions in a single turn */
  max_followup_questions: z.number().int().min(0), 
  /** Preferred response format (e.g., 'bullet_points', 'prose') */
  format: z.string() 
});

/**
 * @schema ContextStateSchema
 * @description Sanitized, anonymized summary of the dialogue for reasoning.
 */
const ContextStateSchema = z.object({
  /** A clean, privacy-safe text summary of the relevant dialogue context */
  anonymized_summary: z.string(), 
  /** Boolean: Indicates if a specific text excerpt was explicitly approved */
  included_raw_excerpt: z.boolean(), 
  /** Metadata regarding context retrieval (e.g., 'vector_search_summary') */
  retrieval_notes_included: z.boolean() 
});

// ----------------------------------------------------------------------------
// 3. MASTER SCHEMA (The Root Object)
// ----------------------------------------------------------------------------

/**
 * @schema EQStateSchema
 * @description The complete, top-level validated payload emitted by the EQ Engine.
 * This is the object that travels through the MCP bridge.
 */
export const EQStateSchema = z.object({
  /** Version of the schema for compatibility checking */
  schema_version: z.string(), 
  /** Metadata about the ephemeral session */
  session: z.object({
    ephemeral_session_id: z.string().uuid(), 
    timestamp_local: z.string().datetime(), 
    device_processing_only: z.boolean() 
  }), 
  /** The emotional dimension of the state */
  affect: AffectStateSchema, 
  /** The goal-oriented dimension of the state */
  intent: IntentStateSchema, 
  /** The safety dimension of the state */
  risk: RiskStateSchema, 
  /** The privacy and sanitization dimension of the state */
  privacy: PrivacyStateSchema, 
  /** The behavioral dimension of the state */
  response_policy: ResponsePolicySchema, 
  /** The contextual dimension of the state */
  context: ContextStateSchema 
});

/**
 * @type {z.infer<typeof EQStateSchema>}
 * @description The TypeScript type inferred from the master schema.
 * Use this type for all functions handling the EQ State.
 */
export type EQState = z.infer<typeof EQStateSchema>;
