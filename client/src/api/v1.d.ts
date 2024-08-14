/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */


export interface paths {
  "/counter/{key}": {
    /** Fetch the current value of the counter. */
    get: operations["get_counter"];
    /** Update the current value of the counter. */
    put: operations["put_counter"];
  };
}

export type webhooks = Record<string, never>;

export interface components {
  schemas: {
    /** @description `CounterValue` represents the value of the API's counter, either as the response to a GET request to fetch the counter or as the body of a PUT request to update the counter. */
    readonly CounterValue: {
      /** Format: uint32 */
      readonly counter?: number | null;
    };
    /** @description Error information from a response. */
    readonly Error: {
      readonly error_code?: string;
      readonly message: string;
      readonly request_id: string;
    };
  };
  responses: {
    /** @description Error */
    readonly Error: {
      content: {
        readonly "application/json": components["schemas"]["Error"];
      };
    };
  };
  parameters: never;
  requestBodies: never;
  headers: never;
  pathItems: never;
}

export type $defs = Record<string, never>;

export type external = Record<string, never>;

export interface operations {

  /** Fetch the current value of the counter. */
  get_counter: {
    parameters: {
      path: {
        key: string;
      };
    };
    responses: {
      /** @description successful operation */
      200: {
        content: {
          readonly "application/json": components["schemas"]["CounterValue"];
        };
      };
      "4XX": components["responses"]["Error"];
      "5XX": components["responses"]["Error"];
    };
  };
  /** Update the current value of the counter. */
  put_counter: {
    parameters: {
      path: {
        key: string;
      };
    };
    readonly requestBody: {
      readonly content: {
        readonly "application/json": components["schemas"]["CounterValue"];
      };
    };
    responses: {
      /** @description resource updated */
      204: {
        content: never;
      };
      "4XX": components["responses"]["Error"];
      "5XX": components["responses"]["Error"];
    };
  };
}
