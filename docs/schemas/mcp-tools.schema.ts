/**
 * @file mcp-tools.schema.ts
 * @description Definitive TypeScript/Zod definitions for the EQ Gateway MCP (Model Context Protocol) tools.
 * These definitions represent the "Top" of the intelligence stack—the interface exposed to the Large AI.
 * Every tool is defined with strict input/output schemas to prevent contract drift between the local
 * EQ Engine and the cloud-based Large AI.
 * 
 * VERSION: 0.2
 * STATUS: Finalized / Strict Implementation
 * 
 * ============================================================================
 * COMPREHENSIVE VARIABLE & TYPE SUMMARY
 * ============================================================================
 * 
 * --- IMPORTS (Re-exported Types from eq-state.schema.ts) ---
 * [EQState] : The complete EQ state payload type (inferred from EQStateSchema).
 * 
 * --- INPUT SCHEMAS (Zod Validation Objects) ---
 * [GetEqStateInputSchema]              : detail_level (minimal | standard | extended).
 * [GetAnonymizedContextInputSchema]    : task_scope (string), max_tokens (int 100-1500).
 * [RequestApprovalInputSchema]         : reason (string), excerpt_preview (string) for user UI prompt.
 * 
 * --- OUTPUT SCHEMAS (Zod Validation Objects) ---
 * [GetEqStateOutputSchema]             : eq_state (EQState) + captured_at (ISO-8601).
 * [GetAnonymizedContextOutputSchema]   : anonymized_summary + metadata (token_count, confidence).
 * [GetResponsePolicyOutputSchema]      : policy object using shared controlled vocabularies.
 * [RequestApprovalOutputSchema]        : status (approved | denied | timeout) + optional excerpt.
 * 
 * --- TOOL REGISTRY ---
 * [MCP_TOOLS] : Array of 4 tool definitions serialized to the MCP protocol.
 * [MCPTools]  : TypeScript type inferred from MCP_TOOLS array.
 * 
 * ============================================================================
 */

import { z } from 'zod';
import { EQState, ResponseTone } from './eq-state.schema'; // Importing the Source of Truth

// ----------------------------------------------------------------------------
// 1. TOOL INPUT SCHEMAS
// ----------------------------------------------------------------------------

/**
 * @schema GetEqStateInput
 * @description Input parameters for the `get_eq_state` tool.
 */
export const GetEqStateInputSchema = z.object({
  /** 
   * Determines the verbosity of the EQ State returned.
   * 'minimal': Only primary affects and risk level.
   * 'standard': Full EQ State including intent and privacy.
   * 'extended': Full EQ State plus detailed evidence/rationale.
   */
  detail_level: z.enum(['minimal', 'standard', 'extended'])
});

/**
 * @schema GetAnonymizedContextInput
 * @description Input parameters for the `get_anonymized_context` tool.
 */
export const GetAnonymizedContextInputSchema = z.object({
  /** A brief description of the task the AI is performing (used for context weighting) */
  task_scope: z.string(),
  /** Maximum number of tokens to return in the summary (100 - 1500) */
  max_tokens: z.number().int().min(100).max(1500)
});

/**
 * @schema RequestApprovalInput
 * @description Input parameters for the `request_user_approval_for_raw_excerpt` tool.
 * This tool triggers a high-priority interrupt in the mobile UI.
 */
export const RequestApprovalInputSchema = z.object({
  /** The justification provided to the user for why this raw data is needed */
  reason: z.string(),
  /** A safe, truncated preview of the text being requested for approval */
  excerpt_preview: z.string()
});

// ----------------------------------------------------------------------------
// 2. TOOL OUTPUT SCHEMAS (The "Return" values seen by the Large AI)
// ----------------------------------------------------------------------------

/**
 * @schema GetEqStateOutput
 * @description The structured response returned by the `get_eq_state` tool.
 */
export const GetEqStateOutputSchema = z.object({
  /** The structured EQ state payload */
  eq_state: EQState,
  /** Timestamp of when the state was captured */
  captured_at: z.string().datetime()
});

/**
 * @schema GetAnonymizedContextOutput
 * @description The response containing the sanitized textual summary.
 */
export const GetAnonymizedContextOutputSchema = z.object({
  /** The cleaned, summarized text string */
  anonymized_summary: z.string(),
  /** Metadata about the summary generation (e.g., compression ratio, confidence) */
  metadata: z.object({
    /** Number of tokens in the returned summary (1 to max_tokens requested) */
    token_count: z.number().int().min(1),
    /** Model's confidence in the anonymization/summary quality (0.0 to 1.0) */
    confidence: z.number().min(0).max(1)
  })
});

/**
 * @schema GetResponsePolicyOutput
 * @description The response providing stylistic constraints passed through the MCP bridge.
 * All categorical fields use the shared controlled vocabularies from eq-state.schema.ts
 * to ensure zero ambiguity between the EQ Engine and the Large AI.
 */
export const GetResponsePolicyOutputSchema = z.object({
  /** The response policy object extracted from the EQ Engine's local inference */
  policy: z.object({
    /** Desired tone of the AI's response (shared enum: calm_direct, gentle_direct, etc.) */
    tone: ResponseTone,
    /** Degree of warmth/empathy in the response (0.0 to 1.0) */
    warmth: z.number().min(0).max(1),
    /** Degree of directness/brevity (0.0 to 1.0) */
    directness: z.number().min(0).max(1),
    /** Target response length (shared with eq-state schema) */
    length: z.enum(['short', 'medium', 'long']),
    /** Hard limit on the number of follow-up questions the AI may ask in a single turn */
    max_followup_questions: z.number().int().min(0)
  })
});

/**
 * @schema RequestApprovalOutput
 * @description The response after a user interaction.
 */
export const RequestApprovalOutputSchema = z.object({
  /** The final decision made by the user */
  status: z.enum(['approved', 'denied', 'timeout']),
  /** If approved, the actual raw text segment allowed to be shared */
  authorized_excerpt: z.string().optional(),
  /** Timestamp of the user decision */
  decision_timestamp: z.string().datetime()
});

// ----------------------------------------------------------------------------
// 3. MASTER MCP TOOLSET DEFINITION
// ----------------------------------------------------------------------------

/**
 * @constant MCP_TOOLS
 * @description The complete registry of tools available to the Large AI via the MCP protocol.
 * Each entry defines a name, description, input schema, and output schema.
 * This registry is serialized and served to the LLM at session initialization.
 * 
 * Order matters: tools are listed by typical call frequency (most frequent first).
 */
export const MCP_TOOLS = [
  {
    name: 'get_eq_state',
    description: 'Returns the current privacy-preserving emotional state and response policy inferred on-device.',
    input_schema: GetEqStateInputSchema,
    output_schema: GetEqStateOutputSchema
  },
  {
    name: 'get_anonymized_context',
    description: 'Returns a sanitized summary of the user\'s relevant context without raw private identifiers.',
    input_schema: GetAnonymizedContextInputSchema,
    output_schema: GetAnonymizedContextOutputSchema
  },
  {
    name: 'get_response_policy',
    description: 'Returns only the recommended tone, length, and directness policy for the response.',
    input_schema: z.object({}), // No inputs required; policy is derived from current EQ Engine state
    output_schema: GetResponsePolicyOutputSchema
  },
  {
    name: 'request_user_approval_for_raw_excerpt',
    description: 'Triggers a mandatory user UI prompt to approve sharing a specific text excerpt with the cloud.',
    input_schema: RequestApprovalInputSchema,
    output_schema: RequestApprovalOutputSchema
  }
] as const;

/**
 * @type {ReadonlyArray<{readonly name: string; readonly description: string; readonly input_schema: z.ZodType; readonly output_schema: z.ZodType}>}
 * @description Strict TypeScript type inferred from the MCP_TOOLS constant array.
 * Using `as const` ensures literal types are preserved for tool names.
 */
export type MCPTools = typeof MCP_TOOLS;

/**
 * @function validateToolInput
 * @description Runtime validation helper for MCP tool inputs.
 * Looks up the tool by name, extracts its input_schema, and parses the provided args.
 * Throws a detailed ZodError if validation fails.
 * 
 * @param toolName - The name of the MCP tool to validate against
 * @param args - The raw input arguments from the Large AI
 * @returns The parsed and type-safe input object
 * 
 * @example
 * const input = validateToolInput('get_eq_state', { detail_level: 'standard' });
 * // input.detail_level is now typed as 'minimal' | 'standard' | 'extended'
 */
export function validateToolInput(toolName: string, args: unknown): unknown {
  const tool = MCP_TOOLS.find(t => t.name === toolName);
  if (!tool) {
    throw new Error(`Unknown MCP tool: "${toolName}". Available tools: ${MCP_TOOLS.map(t => t.name).join(', ')}`);
  }
  return tool.input_schema.parse(args);
}

/**
 * @function validateToolOutput
 * @description Runtime validation helper for MCP tool outputs.
 * Ensures the Rust/KMP engine produces payloads that conform to the expected schema
 * before they are sent to the Large AI. This is a critical safety check.
 * 
 * @param toolName - The name of the MCP tool that produced the output
 * @param data - The raw output data from the EQ Engine
 * @returns The parsed and type-safe output object
 */
export function validateToolOutput(toolName: string, data: unknown): unknown {
  const tool = MCP_TOOLS.find(t => t.name === toolName);
  if (!tool) {
    throw new Error(`Unknown MCP tool: "${toolName}". Cannot validate output.`);
  }
  return tool.output_schema.parse(data);
}
