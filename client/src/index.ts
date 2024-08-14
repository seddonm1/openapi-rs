import createClient from "openapi-fetch";
import type { paths } from "./api/v1";

// A very simple example showing how to call the server using code-generated type-safe code.
const main = async () => {
    const client = createClient<paths>({ baseUrl: "http://127.0.0.1:8080" });

    const {
        data: data0, // only present if 2XX response
        error: error0, // only present if 4XX or 5XX response
    } = await client.GET("/counter/bdfdc549-f507-4405-836b-7901f35a8b0f", {
        params: {},
    });
    console.log(data0, error0);

    const {
        data: data1, // only present if 2XX response
        error: error1, // only present if 4XX or 5XX response
    } = await client.PUT("/counter/bdfdc549-f507-4405-836b-7901f35a8b0f", {
        params: {},
        body: {
            counter: 11
        }
    });
    console.log(data1, error1);

    {
        const {
            data, // only present if 2XX response
            error, // only present if 4XX or 5XX response
        } = await client.GET("/counter/bdfdc549-f507-4405-836b-7901f35a8b0f", {
            params: {},
        });
        console.log(data, error);
    }
};

main();