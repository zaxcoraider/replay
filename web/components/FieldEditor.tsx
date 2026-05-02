"use client";
import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

interface Props {
  value: unknown;
  path: string;
  onChange: (path: string, newValue: unknown) => void;
  modifiedPaths: Set<string>;
  depth?: number;
}

function isEnumValue(v: unknown): v is { variant: string; payload: unknown } {
  if (typeof v !== "object" || v === null) return false;
  const keys = Object.keys(v as object);
  return keys.length === 2 && "variant" in v && "payload" in v;
}

function ModifiedBadge() {
  return (
    <Badge
      variant="outline"
      className="text-[9px] text-yellow-400 border-yellow-700 py-0 h-4 shrink-0"
    >
      edited
    </Badge>
  );
}

export function FieldEditor({ value, path, onChange, modifiedPaths, depth = 0 }: Props) {
  const [expanded, setExpanded] = useState(depth < 2);
  const isModified = modifiedPaths.has(path);

  if (value === null || value === undefined) {
    return <span className="text-zinc-600 text-xs font-mono">null</span>;
  }

  if (typeof value === "boolean") {
    return (
      <span className={`flex items-center gap-1.5 ${isModified ? "bg-yellow-950/30 rounded px-1" : ""}`}>
        <input
          type="checkbox"
          checked={value}
          onChange={(e) => onChange(path, e.target.checked)}
          className="accent-blue-500 cursor-pointer"
        />
        {isModified && <ModifiedBadge />}
      </span>
    );
  }

  if (typeof value === "number") {
    return (
      <span className="flex items-center gap-1">
        <Input
          type="number"
          value={value}
          onChange={(e) => {
            const n = Number(e.target.value);
            if (!isNaN(n)) onChange(path, n);
          }}
          className={`h-6 w-28 text-xs font-mono py-0 px-1.5 bg-zinc-900 ${
            isModified ? "border-yellow-600" : "border-zinc-700"
          }`}
        />
        {isModified && <ModifiedBadge />}
      </span>
    );
  }

  if (typeof value === "string") {
    return (
      <span className="flex items-center gap-1 min-w-0 flex-1">
        <Input
          value={value}
          onChange={(e) => onChange(path, e.target.value)}
          className={`h-6 text-xs font-mono py-0 px-1.5 bg-zinc-900 min-w-0 max-w-[190px] ${
            isModified ? "border-yellow-600" : "border-zinc-700"
          }`}
        />
        {isModified && <ModifiedBadge />}
      </span>
    );
  }

  if (Array.isArray(value)) {
    return (
      <span className="block">
        <button
          onClick={() => setExpanded((e) => !e)}
          className="text-xs text-zinc-400 hover:text-zinc-200 font-mono"
        >
          {expanded ? "▾" : "▸"} [{value.length}]
        </button>
        {expanded &&
          value.map((item, i) => (
            <div key={i} className="flex items-start gap-1.5 mt-0.5 pl-3">
              <span className="text-zinc-600 text-[11px] font-mono pt-0.5 shrink-0">[{i}]</span>
              <FieldEditor
                value={item}
                path={`${path}.${i}`}
                onChange={onChange}
                modifiedPaths={modifiedPaths}
                depth={depth + 1}
              />
            </div>
          ))}
      </span>
    );
  }

  if (isEnumValue(value)) {
    return (
      <span className="flex items-center gap-2">
        <span className="text-purple-400 text-xs font-mono shrink-0">{value.variant}</span>
        {value.payload !== null && value.payload !== undefined && (
          <FieldEditor
            value={value.payload}
            path={`${path}.payload`}
            onChange={onChange}
            modifiedPaths={modifiedPaths}
            depth={depth}
          />
        )}
      </span>
    );
  }

  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    const collapsible = entries.length > 3;
    return (
      <span className="block">
        {collapsible && (
          <button
            onClick={() => setExpanded((e) => !e)}
            className="text-xs text-zinc-400 hover:text-zinc-200 font-mono"
          >
            {expanded ? "▾" : "▸"} {entries.length} fields
          </button>
        )}
        {(expanded || !collapsible) &&
          entries.map(([k, v]) => (
            <div key={k} className="flex items-start gap-1.5 mt-0.5 pl-3">
              <span className="text-zinc-500 text-[11px] font-mono shrink-0 pt-0.5 w-[72px] truncate">
                {k}
              </span>
              <FieldEditor
                value={v}
                path={path ? `${path}.${k}` : k}
                onChange={onChange}
                modifiedPaths={modifiedPaths}
                depth={depth + 1}
              />
            </div>
          ))}
      </span>
    );
  }

  return (
    <span className="text-zinc-400 text-xs font-mono">{String(value)}</span>
  );
}
