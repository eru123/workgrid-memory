// Profile type definitions for WorkGrid Memory

export type ProfileType =
  | "person"
  | "pet"
  | "place"
  | "object"
  | "product"
  | "organization"
  | "job"
  | "idea"
  | "process"
  | "preference"
  | "skill"
  | "client"
  | "custom";

export const PROFILE_TYPES: { value: ProfileType; label: string; icon: string }[] = [
  { value: "person", label: "Person", icon: "👤" },
  { value: "pet", label: "Pet", icon: "🐾" },
  { value: "place", label: "Place", icon: "📍" },
  { value: "object", label: "Object", icon: "📦" },
  { value: "product", label: "Product", icon: "🚀" },
  { value: "organization", label: "Organization", icon: "🏢" },
  { value: "job", label: "Job / Role", icon: "💼" },
  { value: "idea", label: "Idea", icon: "💡" },
  { value: "process", label: "Process", icon: "🔄" },
  { value: "preference", label: "Preference", icon: "⚙️" },
  { value: "skill", label: "Skill / Instruction", icon: "🎯" },
  { value: "client", label: "Client", icon: "🤝" },
  { value: "custom", label: "Custom Type", icon: "📋" },
];

export type SensitivityLevel = "public" | "internal" | "private" | "sensitive" | "secret";

export const SENSITIVITY_LEVELS: { value: SensitivityLevel; label: string; description: string }[] = [
  { value: "public", label: "Public", description: "Visible to anyone" },
  { value: "internal", label: "Internal", description: "Visible within the app" },
  { value: "private", label: "Private", description: "Not exposed by default" },
  { value: "sensitive", label: "Sensitive", description: "Requires explicit opt-in" },
  { value: "secret", label: "Secret", description: "Never exposed" },
];

export type RelationshipType =
  | "related_to"
  | "owned_by"
  | "works_at"
  | "friend_of"
  | "spouse_of"
  | "pet_of"
  | "located_at"
  | "prefers"
  | "uses"
  | "belongs_to"
  | "teaches"
  | "applies_to"
  | "relevant_to_workspace"
  | "conflicts_with"
  | "supersedes";

export interface ProfileAttribute {
  id: string;
  profileId: string;
  key: string;
  valueJson: string;
  sensitivity?: SensitivityLevel;
  source?: string;
  confidence: number;
}

export interface ProfileInstruction {
  id: string;
  profileId: string;
  name: string;
  triggerTerms: string[];
  rules: string[];
  examples?: string[];
  antiPatterns?: string[];
  priority: number;
  enabled: boolean;
}

export interface ProfileAsset {
  id: string;
  profileId: string;
  assetType: "image" | "document" | "note" | "link";
  localPath: string;
  hash?: string;
  description?: string;
  sensitivity: SensitivityLevel;
}

export interface ProfileRelationship {
  id: string;
  fromProfileId: string;
  toProfileId: string;
  relationshipType: RelationshipType;
  confidence: number;
  source?: string;
}

export interface ProfileWorkspaceLink {
  id: string;
  profileId: string;
  workspaceId: string;
  relevance?: string;
  enabled: boolean;
}
