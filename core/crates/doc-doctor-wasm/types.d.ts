/**
 * TypeScript type definitions for @doc-doctor/wasm
 *
 * These types mirror the Rust WASM bindings for type-safe usage
 * in JavaScript/TypeScript applications.
 */

/** WASM module initialization */
export default function init(): Promise<void>;

/** Main Doc-Doctor interface */
export class DocDoctor {
  constructor();

  /** Parse a markdown document and return L1 properties as JSON */
  parseDocument(content: string): string;

  /** Analyze a document (parse + calculate dimensions) */
  analyzeDocument(content: string): string;

  /** Validate a document against the J-Editorial schema */
  validateDocument(content: string, strict: boolean): string;

  /** Calculate health score */
  calculateHealth(refinement: number, stubsJson: string): number;

  /** Calculate usefulness for an audience */
  calculateUsefulness(refinement: number, audience: string): string;

  /** Calculate vector physics for a stub */
  calculateVectorPhysics(stubJson: string, contextJson: string): string;

  /** Calculate state dimensions for properties */
  calculateDimensions(propertiesJson: string): string;

  /** Get audience gate values */
  getAudienceGates(): string;

  /** Parse stubs from JSON array */
  parseStubs(stubsJson: string): string;

  /** Get the version string */
  version(): string;

  /** Get the parser format identifier */
  formatId(): string;
}

// ============================================================================
// JSON Response Types
// ============================================================================

/** Document analysis result */
export interface AnalysisResult {
  success: boolean;
  error?: string;
  properties?: L1Properties;
  dimensions?: StateDimensions;
  warnings: string[];
}

/** L1 Intrinsic Properties */
export interface L1Properties {
  uid?: string;
  title?: string;
  refinement: number;
  audience: AudienceType;
  origin: OriginType;
  form: FormType;
  stubs: Stub[];
  tags: string[];
  aliases: string[];
}

/** Stub representing a gap or issue */
export interface Stub {
  type: string;
  description: string;
  stubForm: StubFormType;
  priority: PriorityType;
  origin: StubOriginType;
  anchor?: string;
  urgency?: number;
  impact?: number;
  complexity?: number;
  inlineAnchors: string[];
  assignees: string[];
  participants: string[];
  references: string[];
  dependencies: string[];
}

/** State dimensions */
export interface StateDimensions {
  health: number;
  usefulness: Usefulness;
  trustLevel: number;
  freshness: number;
  complianceFit: number;
  coverageFit: number;
}

/** Usefulness assessment */
export interface Usefulness {
  margin: number;
  isUseful: boolean;
  audience: AudienceType;
  refinement: number;
  gate: number;
}

/** Vector physics for stub prioritization */
export interface VectorPhysics {
  potentialEnergy: number;
  frictionCoefficient: number;
  editorialVelocity: number;
  magnitude: number;
}

/** Audience gate thresholds */
export interface AudienceGates {
  personal: number;
  internal: number;
  trusted: number;
  public: number;
}

/** Validation result */
export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

/** Validation error */
export interface ValidationError {
  message: string;
  path?: string;
  line?: number;
  column?: number;
}

/** Validation warning */
export interface ValidationWarning {
  message: string;
  path?: string;
  suggestion?: string;
}

/** Context for vector physics calculations */
export interface StubContext {
  editorialVelocity?: number;
  hasExternalDependencies: boolean;
  ageDays?: number;
  hasControversy: boolean;
}

// ============================================================================
// Enum Types
// ============================================================================

/** Audience levels */
export type AudienceType = 'personal' | 'internal' | 'trusted' | 'public';

/** Document origin */
export type OriginType =
  | 'question'
  | 'requirement'
  | 'insight'
  | 'dialogue'
  | 'curiosity'
  | 'human'
  | 'machine';

/** Document form */
export type FormType =
  | 'transient'
  | 'developing'
  | 'stable'
  | 'evergreen'
  | 'canonical';

/** Stub form (severity) */
export type StubFormType =
  | 'transient'
  | 'persistent'
  | 'blocking'
  | 'structural';

/** Priority level */
export type PriorityType = 'low' | 'medium' | 'high' | 'critical';

/** Stub origin */
export type StubOriginType =
  | 'author_identified'
  | 'peer_surfaced'
  | 'qa_detected'
  | 'user_reported'
  | 'system_generated'
  | 'external_cited';

/** Vector family */
export type VectorFamilyType =
  | 'retrieval'
  | 'computation'
  | 'synthesis'
  | 'creation'
  | 'structural';

// ============================================================================
// Convenience Functions (for typed parsing)
// ============================================================================

/**
 * Parse analysis result from JSON
 * @param json - JSON string from analyzeDocument()
 */
export function parseAnalysisResult(json: string): AnalysisResult;

/**
 * Parse properties from JSON
 * @param json - JSON string from parseDocument()
 */
export function parseProperties(json: string): L1Properties;

/**
 * Parse validation result from JSON
 * @param json - JSON string from validateDocument()
 */
export function parseValidationResult(json: string): ValidationResult;

/**
 * Parse usefulness from JSON
 * @param json - JSON string from calculateUsefulness()
 */
export function parseUsefulness(json: string): Usefulness;

/**
 * Parse dimensions from JSON
 * @param json - JSON string from calculateDimensions()
 */
export function parseDimensions(json: string): StateDimensions;

/**
 * Parse audience gates from JSON
 * @param json - JSON string from getAudienceGates()
 */
export function parseAudienceGates(json: string): AudienceGates;
