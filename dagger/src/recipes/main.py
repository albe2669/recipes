import dagger
from dagger import dag, function, object_type


@object_type
class Recipes:
    def build_book_builder(self, source: dagger.Directory) -> dagger.File:
        return (
            dag.container()
            .from_("rust:alpine")
            .with_exec(
                ["apk", "add", "--update", "--no-cache", "musl-dev", "alpine-sdk"]
            )
            .with_directory("/app", source)
            .with_workdir("/app")
            .with_exec(["cargo", "build", "--release"])
            # .with_mounted_cache("/app/target", dag.cache_volume("book-builder-target"))
            .file("/app/target/release/book-builder")
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
        book_builder = await self.build_book_builder(source.directory("book-builder"))

        latex_env = (
            self.build_latex_env()
            .with_directory("/app", source)
            .with_workdir("/app")
            .with_file("/app/book-builder", book_builder, permissions=0o755)
            .with_exec(["./book-builder", "-l", "latex", "-o", "/app/out", "./Dinner"])
            .with_workdir("/app/out")
            .with_exec(["tectonic", "main.tex"])
        )

        return latex_env.file("/app/out/main.pdf")
