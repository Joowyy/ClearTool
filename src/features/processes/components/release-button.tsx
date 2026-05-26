import { useMutation } from "@tanstack/react-query";
import { releaseCaches } from "../../../api";
import { useNavigate } from "react-router-dom";
import { Sparkles } from "lucide-react";
import { Button } from "../../../components/ui/button";
import { toast } from "../../../lib/toast";

export function ReleaseButton() {
  const navigate = useNavigate();
  const release = useMutation({
    mutationFn: releaseCaches,
    onSuccess: (report) => {
      toast.success(`${report.closedCount} apps cerradas`, {
        description: "Redirigiendo a Cach\u00e9...",
      });
      setTimeout(() => navigate("/cache"), 1500);
    },
    onError: (err) => toast.error("Error cerrando apps", err),
  });

  return (
    <Button
      variant="outline"
      onClick={() => {
        if (confirm(
          "Esto cerrar\u00e1 gr\u00e1cilmente Chrome, Edge, Spotify, Discord, Claude y similares. \u00bfContinuar?"
        )) release.mutate();
      }}
      disabled={release.isPending}
    >
      <Sparkles className="h-4 w-4 mr-2" />
      {release.isPending ? "Cerrando..." : "Liberar para limpiar"}
    </Button>
  );
}
