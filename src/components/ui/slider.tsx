import type { InputHTMLAttributes } from "react";

import { cn } from "../../lib/utils";

type SliderProps = Omit<InputHTMLAttributes<HTMLInputElement>, "onChange" | "type" | "value"> & {
  onValueChange?: (value: number) => void;
  value: number;
};

export function Slider({
  className,
  max = 100,
  min = 1,
  onValueChange,
  step = 1,
  value,
  ...props
}: SliderProps) {
  const minNumber = Number(min);
  const maxNumber = Number(max);
  const valueNumber = Number(value);
  const progress =
    Number.isFinite(minNumber) &&
    Number.isFinite(maxNumber) &&
    Number.isFinite(valueNumber) &&
    maxNumber > minNumber
      ? Math.min(
          100,
          Math.max(0, ((valueNumber - minNumber) / (maxNumber - minNumber)) * 100),
        )
      : 0;

  return (
    <span className="ui-slider-shell">
      <span className="ui-slider-track" aria-hidden="true">
        <span className="ui-slider-progress" style={{ width: `${progress}%` }} />
      </span>
      <input
        className={cn("ui-slider", className)}
        max={max}
        min={min}
        onChange={(event) => onValueChange?.(Number(event.currentTarget.value))}
        step={step}
        type="range"
        value={value}
        {...props}
      />
    </span>
  );
}
