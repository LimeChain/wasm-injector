hosts = ["polkadot", "kagome"]

# Add locally build or downloaded adapters, testers and hosts to PATH
ENV["PATH"] *= ":$(pwd())/bin"

tests = readdir(testDir)
for host in hosts
    withenv("ZOMBIENET_DEFAULT_START_COMMAND" => host) do
        for test in tests
            # There shouldn't be any other files in the tests directory. Just to be safe.
            if test[end-5:end] == ".zndsl"
                command = `zombienet -p native test ./tests/$test`
                println("Running test command[$host]: ", command)
                try
                    run(command)
                catch e
                    println("Test failed: ", test)
                    println(e)
                end
            end
        end
    end
end
