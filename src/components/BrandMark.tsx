import type { SVGProps } from "react";

export function BrandMark({ className, ...props }: SVGProps<SVGSVGElement>) {
  return (
    <svg
      viewBox="0 0 512 512"
      className={className}
      role="img"
      aria-label="Janus 双面之门"
      {...props}
    >
      <path d="M152 386V252c0-77 50-125 90-126" fill="none" stroke="currentColor" strokeWidth="34" strokeLinecap="round" />
      <path d="M360 386V252c0-77-50-125-90-126" fill="none" stroke="currentColor" strokeWidth="34" strokeLinecap="round" />
      <circle cx="256" cy="126" r="19" fill="var(--gold)" />
    </svg>
  );
}
