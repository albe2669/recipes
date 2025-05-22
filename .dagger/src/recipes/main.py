import dagger
from dagger import dag, function, object_type


@object_type
class Recipes:
    def get_cooklatex(self) -> dagger.File:
        return (
            dag.container()
            .from_("rust:alpine")
            .with_exec(["apk", "add", "--update", "--no-cache", "wget"])
            .with_workdir("/app")
            .with_exec(
                [
                    "wget",
                    "https://github.com/albe2669/cooklatex/releases/latest/download/cooklatex",
                    "-O",
                    "cooklatex",
                ]
            )
            .file("/app/cooklatex")
        )

    def build_latex_env(self) -> dagger.Container:
        return (
            dag.container()
            .from_("alpine:latest")
            .with_exec(["apk", "add", "--update", "--no-cache", "tectonic"])
        )

    @function
    async def get_pdf(
        self,
        source: dagger.Directory,
    ) -> dagger.File:
        latex_env = self.build_latex_env()
        cooklatex = await self.get_cooklatex()

        latex_env = (
            self.build_latex_env()
            .with_directory("/app", source)
            .with_workdir("/app")
            .with_file("/app/cooklatex", cooklatex, permissions=0o755)
            .with_exec(["./cooklatex", "-l", ".template", "-o", "/app/out", "./Dinner"])
            .with_workdir("/app/out")
            .with_exec(["tectonic", "main.tex"])
        )

        return latex_env.file("/app/out/main.pdf")
