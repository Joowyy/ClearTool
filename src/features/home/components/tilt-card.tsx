import { motion, useMotionValue, useTransform, useSpring } from "framer-motion";
import { type ReactNode, type MouseEvent } from "react";
import { cn } from "../../../lib/utils";

interface TiltCardProps {
  children: ReactNode;
  className?: string;
  intensity?: number; // 0-12 grados
}

/// Tarjeta con efecto tilt 3D que sigue al cursor. Pensada para envolver
/// cualquier widget del Dashboard. Se desactiva en pantallas touch para
/// no perjudicar accesibilidad.
export function TiltCard({ children, className, intensity = 6 }: TiltCardProps) {
  const x = useMotionValue(0);
  const y = useMotionValue(0);
  const rotateX = useSpring(useTransform(y, [-50, 50], [intensity, -intensity]), {
    stiffness: 220,
    damping: 22,
  });
  const rotateY = useSpring(useTransform(x, [-50, 50], [-intensity, intensity]), {
    stiffness: 220,
    damping: 22,
  });

  const handleMove = (e: MouseEvent<HTMLDivElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    x.set(e.clientX - r.left - r.width / 2);
    y.set(e.clientY - r.top - r.height / 2);
  };
  const handleLeave = () => {
    x.set(0);
    y.set(0);
  };

  return (
    <motion.div
      onMouseMove={handleMove}
      onMouseLeave={handleLeave}
      style={{
        rotateX,
        rotateY,
        transformStyle: "preserve-3d",
        perspective: 1200,
      }}
      className={cn("glass glass-hover p-5", className)}
    >
      {children}
    </motion.div>
  );
}
