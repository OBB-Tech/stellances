import { Test, TestingModule } from '@nestjs/testing';
import { AppController } from './app.controller';
import { AppService } from './app.service';

describe('AppController', () => {
  let appController: AppController;

  beforeEach(async () => {
    const app: TestingModule = await Test.createTestingModule({
      controllers: [AppController],
      providers: [AppService],
    }).compile();

    appController = app.get<AppController>(AppController);
  });

  it('getHello() returns a non-empty string (health check)', () => {
    const result = appController.getHello();
    expect(typeof result).toBe('string');
    expect(result.length).toBeGreaterThan(0);
  });
});
